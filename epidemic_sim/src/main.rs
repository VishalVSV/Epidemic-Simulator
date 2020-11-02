use rand::Rng;

use std::{time};
use std::fs::File;
use std::io::{BufWriter,Write};

#[derive(Debug)]
enum Person {
	Susceptible,				//Are at risk of contraction Normal people
	Infectious(u32,bool,bool),	//Are capable of transmitting the disease (Hours since infected,To Remove,Increment hospitalisation period[i.e Are they hospitalized?])
	Exposed(u32),				//Are infected but not infectious (Hours since infected)
	Resistant,					//Are no longer able to be infected
	Dead
}

struct Simulation {
	pub susceptible: Vec<Person>,	//Population of simulation set
	pub exposed: Vec<Person>,
	pub infected: Vec<Person>,
	resistant: Vec<Person>,
	dead: Vec<Person>,
	
	random: rand::ThreadRng,		//Random Number generator used for the simulation
	n: f64,							//Variable required to calculate avg BRN
	
	time_delta: u32,				//Time after each cycle is calculated in hours (Every how much time should it update)
	pub time_since_start: usize,	//Time since start of simulation in hours
	infectivity: f64,				//The probabilty that a susceptible person exposed to an infectious person is infected
	contact_no: usize,				//Number of people that each people come into contact with per delta time
	incubation_period: f64,			//The time taken for an exposed person to turn infectious
	fatality_rate: f64,				//The probabilty that an infected person dies
	avg_hospitalisation_period: f64,//The average time taken for an infected person to become resistant
	reinfection_rate: f64,			//Chance that a cured person is able to infected
	curabilty: f64,					//Chance that a curable person is cured per delta time
	hospitalisation_rate: f64,		//Chance that an infected person is hospitalized
	brn: f64,						//Basic Reproduction Number
	daily_infections: usize,		//Daily infections
	daily_deaths: usize				//Daily deaths
}

impl Simulation {
	#[allow(dead_code)]
	pub fn new(population_size: usize,infectivity: f64,contact_no: usize,incubation_period: f64,fatality_rate: f64,avg_hospitalisation_period: f64,reinfection_rate: f64,curabilty: f64,hospitalisation_rate: f64) -> Simulation { //Unused in actual code
		let mut susceptible = Vec::with_capacity(population_size);
		for _ in 0..population_size {
			susceptible.push(Person::Susceptible);
		}
	
		Simulation {
			susceptible,
			exposed: Vec::new(),
			infected: Vec::new(),
			resistant: Vec::new(),
			dead: Vec::new(),
			
			random: rand::thread_rng(),
			
			time_delta: 24,
			time_since_start: 0,
			infectivity,
			contact_no,
			incubation_period,
			fatality_rate,
			avg_hospitalisation_period,
			reinfection_rate,
			curabilty,
			brn: 0.0,
			n: 1.0,
			hospitalisation_rate,
			daily_infections:0,
			daily_deaths:0
		}
	}
	
	pub fn from_file(path: &str) -> Simulation { //Make a simulation and configure it using a disease file.
		let contents = std::fs::read_to_string(path).unwrap();
		
		let mut rand = rand::thread_rng();
		
		let mut population_num = 0;
		let mut infected_num = 0;
		let mut infectivity = 0.0;
		let mut contact_no = 0;
		let mut incubation_period = 0.0;
		let mut fatality_rate = 0.0;
		let mut avg_hospitalisation_period = 0.0;
		let mut reinfection_rate = 0.0;
		let mut curabilty = 0.0;
		let mut hospitalisation_rate = 0.0;
		
		let mut checksum: usize = 0;
		
		for l in contents.lines() {
			let line = l.trim();
			if line.contains("=") {
				let data = line.split('=').nth(1).unwrap().trim();
				if line.starts_with("population") {
					population_num = data.parse().unwrap();
					checksum += 1;
				}
				else if line.starts_with("infected") {
					infected_num = data.parse().unwrap();
					checksum += 1;
				}
				else if line.starts_with("infectivity") {
					infectivity = data.parse().unwrap();
					checksum += 1;
				}
				else if line.starts_with("contact_no") {
					contact_no = data.parse().unwrap();
					checksum += 1;
				}
				else if line.starts_with("incubation_period") {
					incubation_period = data.parse().unwrap();
					checksum += 1;
				}
				else if line.starts_with("fatality_rate") {
					fatality_rate = data.parse().unwrap();
					checksum += 1;
				}
				else if line.starts_with("avg_hospitalisation_period") {
					avg_hospitalisation_period = data.parse().unwrap();
					checksum += 1;
				}
				else if line.starts_with("reinfection_rate") {
					reinfection_rate = data.parse().unwrap();
					checksum += 1;
				}
				else if line.starts_with("curability") {
					curabilty = data.parse().unwrap();
					checksum += 1;
				}
				else if line.starts_with("hospitalisation_rate") {
					hospitalisation_rate = data.parse().unwrap();
					checksum += 1;
				}				
			}
		}
		
		assert_eq!(checksum,10,"All parameters not found in disease file. Only found {} entries!",checksum);
		
		// Initialize the vectors with people
		//===============================================================
		let mut susceptible = Vec::new(); 
		for _ in 0..(population_num - infected_num) {
			susceptible.push(Person::Susceptible);
		}
		
		let mut infected = Vec::new();
		for _ in 0..infected_num {
			infected.push(Person::Infectious(0,false,rand.gen::<bool>()));
		}
		//===============================================================
		
		Simulation {
			susceptible,
			infected,
			exposed: Vec::new(),
			dead: Vec::new(),
			resistant: Vec::new(),
			
			random: rand,
			time_delta: 24,
			time_since_start: 0,
			infectivity,
			contact_no,
			incubation_period,
			fatality_rate,
			avg_hospitalisation_period,
			reinfection_rate,
			curabilty,
			brn: 0.0,
			n: 1.0,
			hospitalisation_rate,
			daily_deaths:0,
			daily_infections:0
		}
	}
}

impl Simulation {
	pub fn cycle(&mut self) {
		//Reset daily variables
		self.daily_deaths = 0;
		self.daily_infections = 0;
	
		//Increment the world clock
		self.time_since_start += self.time_delta as usize;
	
		//Infections - Each Infected person infects more people
		let mut i: usize = 0;		
		
		'outer: 
		for _ in &self.infected {
			//The number of people that a person will interact with in this cycle
			let new_contact_no: usize = self.random.gen_range(0,self.contact_no + 1);
			
			//Total number of people this person infects; Is used to calculate Basic Reproduction Number
			let mut num_of_infected: f64 = 0.0;
			
			//Loop through and find people susceptible to infection
			for _ in 0..new_contact_no {
				//Make sure we haven't run out of susceptible people
				if i < self.susceptible.len() {
				
					//Even if a susceptible person interacts with an infected person there is only a chance that they will be infected
					let chance = self.random.gen::<f64>();
					
					if self.infectivity > chance {//Person has been infected
						num_of_infected += 1.0;
						self.susceptible.swap_remove(i);
						self.exposed.push(Person::Exposed(0));
					}
					else {
						i += 1;
					}
				}
				else {//Ran out of susceptible people i.e everyone has interacted with someone [Can be improved as people can interact with multiple people but is not worth the technical complexity]
					
					self.daily_infections += num_of_infected as usize;
					
					//Calcuate the average BRN up to this date [Running average formula]
					self.brn = (((self.n-1.0) * self.brn) + num_of_infected)/self.n;
					self.n += 1.0;
					break 'outer;
				}
			}
			
			self.daily_infections += num_of_infected as usize;
			self.brn = (((self.n-1.0) * self.brn) + num_of_infected)/self.n;
			self.n += 1.0;
		}
		
		//=======================================================
		
		
		//Increment time since infected==========================
		for p_exposed in self.exposed.iter_mut() {
			if let Person::Exposed(ref mut time_since_infected) = p_exposed {
				*time_since_infected += self.time_delta;
			}
		}
		//=======================================================
		
		//Remove exposed who have passed the incubation period and turn them into infected===
		let incubation_period = self.incubation_period;
		
		for p_exposed in &self.exposed {
			if let Person::Exposed(time_since_infected) = p_exposed {
			
				//If the person has been exposed for longer than the incubation period they can now infect people
				if (*time_since_infected as f64) >= incubation_period {
					self.infected.push(Person::Infectious(*time_since_infected,false,false));
				}
			}	
		}
		
		self.exposed.retain(|p_exposed| {
				if let Person::Exposed(time_since_infected) = p_exposed {
					return !((*time_since_infected as f64) >= incubation_period);
				}
				false //Unreachable
			}
		);
		//================================================================================
		
		//If an infected has survived hospitaliztion period then there is a chance to turn them into a resistant person===
		let avg_hospitalisation_period = self.avg_hospitalisation_period;
		
		//Loop through all infected people
		for (_,p_infected) in self.infected.iter_mut().enumerate() {
			if let Person::Infectious(ref mut time_since_infected,ref mut to_remove,ref mut hospitalized) = p_infected {
			
				//If the person is hospitalised i.e has been admitted to a medical instituition
				if *hospitalized {
					//Increment the time spent in hospitals
					*time_since_infected += self.time_delta;
					
					//If they have spent long enough in a hospital they have a chance of being cured
					if (*time_since_infected as f64) >= avg_hospitalisation_period {
						let cure_chance = self.random.gen::<f64>();
						if cure_chance <= self.curabilty {
							*to_remove = true;
							let chance = self.random.gen::<f64>();
							
							//There is also a chance that the person cured will not gain immunity to the disease
							if chance <= self.reinfection_rate {
								self.exposed.push(Person::Exposed(0));
							}
							else {
								self.resistant.push(Person::Resistant);
							}
						}
					}
				}
				//If they haven't been hospitalised then there is a chance they get admitted
				else {
					let chance = self.random.gen::<f64>();
					if chance <= self.hospitalisation_rate {
						*hospitalized = true;
					}
				}
			}
			else {
				//Unreachable
			}
		}
		
		self.infected.retain(|p_infected| {
			if let Person::Infectious(_,to_remove,_) = p_infected {
				return !to_remove;
			}
			return false;//Unreachable
		});
		
		//=====================================================================================
		
		//Check to see if they are dead=====================================================
		let init_infected = self.infected.len();
		let fatality_rate = self.fatality_rate;
		let mut rand = self.random.clone();
		
		let mut daily_deaths_temp = 0;
		
		self.infected.retain(|_| {
				let chance = rand.gen::<f64>();
				let dead = fatality_rate >= chance;
				
				if dead {
					daily_deaths_temp += 1;
				}
				
				!(dead)//Retain if not dead
			}
		);
		
		self.daily_deaths = daily_deaths_temp;
		
		for _ in 0..(init_infected - self.infected.len()) {
			self.dead.push(Person::Dead);
		}
		//===================================================================================
	}
	
	pub fn stringify(&self) -> String {
		format!("Susceptible: {}            \nExposed: {}            \nInfected: {}            \nResistant: {}            \nDead: {}            \nBRN: {}                     \nDaily_Deaths:{}                      \nDaily_Infections:{}                    \n\n",self.susceptible.len(),self.exposed.len(),self.infected.len(),self.resistant.len(),self.dead.len(),self.brn,self.daily_deaths,self.daily_infections)
	}
	
	pub fn condensed_stringify(&self) -> String {
		format!("{} hrs:-\nSusceptible:{}\nExposed:{}\nInfected:{}\nResistant:{}\nDead:{}\nBRN:{}\nDaily_Deaths:{}\nDaily_Infections:{}\n==============\n",self.time_since_start,self.susceptible.len(),self.exposed.len(),self.infected.len(),self.resistant.len(),self.dead.len(),self.brn,self.daily_deaths,self.daily_infections)
	}
}

fn path_exists(path: &str) -> bool {
	std::fs::metadata(path).is_ok()
}

fn main() {
	//Clear console window
	println!("\x1B[2J");
	println!("\x1B[0;0H");
	
	//Find file name
	let mut i = 1;
	let mut path = format!("./simlog{}.txt",i);
	while path_exists(&path) {
		i += 1;
		path = format!("./simlog{}.txt",i);
	}
	
	//Make the file and open a handle to it
	let f = File::create(&path).expect("Unable to create file");
	
	//Initialize the buffered writer
	let mut writer = BufWriter::new(f);
	
	writer.write_all("simlogfile 1.3\n".to_owned().as_bytes()).expect("Writer couldn't write to file!");
	
	//Create simulation struct
	let mut sim = Simulation::from_file("./disease.txt");
	
	//Store the time that the simulation was started for performance diagnostics
	let now = time::Instant::now();
	loop {
		//Run one cycle of the simulation
		sim.cycle();
		
		//Log to console and file
		println!("{}",sim.stringify());
		writer.write_all(sim.condensed_stringify().as_bytes()).expect("Unable to write to log");
		println!("\x1B[0;0H");
		
		//Check if simulation should stop
		if sim.infected.len() == 0 && sim.exposed.len() == 0 && sim.susceptible.len() == 0 {
			let days = sim.time_since_start / 24;	
			
			//Write the number of says take and flush the writer
			writer.write_all((format!("\n\nDisease took {} days to end",sim.time_since_start / 24)).as_bytes()).expect("Writer couldn't write to file!");
			writer.flush().expect("Unable to flush writer!");
			
			
			//Calcuate average sim speed
			let time_taken = now.elapsed().as_secs();
			println!("Total time taken is {} seconds and each day took {} secs on average to process",time_taken,(time_taken as f64) / (days as f64));
			
			break;
		}
	}
	
}