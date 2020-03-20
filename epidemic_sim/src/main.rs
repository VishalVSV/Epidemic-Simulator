use rand::Rng;

use std::fmt;
use std::{thread,time};
use std::fs::File;
use std::io::{BufWriter,Write};

#[derive(Debug)]
enum Person {
	Susceptible,	//Are at risk of contraction Normal people
	Infectious(u32,bool),//Are capable of transmitting the disease (Hours since infected)
	Exposed(u32),	//Are infected but not infectious (Hours since infected)
	Resistant,		//Are no longer able to be infected
	Dead
}

struct Simulation {
	pub susceptible: Vec<Person>,		//Population of simulation set
	pub exposed: Vec<Person>,
	pub infected: Vec<Person>,
	resistant: Vec<Person>,
	dead: Vec<Person>,
	
	random: rand::ThreadRng,		//Random Number generator used for the simulation
	
	time_delta: u32,				//Time after each cycle is calculated in hours (Every how much time should it update)
	pub time_since_start: usize,	//Time since start of simulation in hours
	infectivity: f64,				//The probabilty that a susceptible person exposed to an infectious person is infected
	contact_no: usize,				//Number of people that each people come into contact with per delta time
	incubation_period: f64,			//The time taken for an exposed person to turn infectious
	fatality_rate: f64,				//The probabilty that an infected person dies
	avg_hospitalisation_period: f64,//The average time taken for an infected person to become resistant
	reinfection_rate: f64,			//Chance that a cured person is able to infected
	curabilty: f64					//Chance that a curable person is cured
}

impl Simulation {
	pub fn new(population_size: usize,infectivity: f64,contact_no: usize,incubation_period: f64,fatality_rate: f64,avg_hospitalisation_period: f64,reinfection_rate: f64,curabilty: f64) -> Simulation {
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
			curabilty
		}
	}
	
	pub fn from_file(path: &str) -> Simulation {
		let contents = std::fs::read_to_string(path).unwrap();
		
		let mut population_num = 0;
		let mut infected_num = 0;
		let mut infectivity = 0.0;
		let mut contact_no = 0;
		let mut incubation_period = 0.0;
		let mut fatality_rate = 0.0;
		let mut avg_hospitalisation_period = 0.0;
		let mut reinfection_rate = 0.0;
		let mut curabilty = 0.0;
		
		for l in contents.lines() {
			let line = l.trim();
			if line.contains("=") {
				let data = line.split('=').nth(1).unwrap().trim();
				if line.starts_with("population") {
					population_num = data.parse().unwrap();
				}
				else if line.starts_with("infected") {
					infected_num = data.parse().unwrap();
				}
				else if line.starts_with("infectivity") {
					infectivity = data.parse().unwrap();
				}
				else if line.starts_with("contact_no") {
					contact_no = data.parse().unwrap();
				}
				else if line.starts_with("incubation_period") {
					incubation_period = data.parse().unwrap();
				}
				else if line.starts_with("fatality_rate") {
					fatality_rate = data.parse().unwrap();
				}
				else if line.starts_with("avg_hospitalisation_period") {
					avg_hospitalisation_period = data.parse().unwrap();
				}
				else if line.starts_with("reinfection_rate") {
					reinfection_rate = data.parse().unwrap();
				}
				else if line.starts_with("curability") {
					curabilty = data.parse().unwrap();
				}
				
			}
		}
		
		let mut susceptible = Vec::new();
		for _ in 0..(population_num - infected_num) {
			susceptible.push(Person::Susceptible);
		}
		
		let mut infected = Vec::new();
		for _ in 0..infected_num {
			infected.push(Person::Infectious(0,false));
		}
		
		Simulation {
			susceptible,
			infected,
			exposed: Vec::new(),
			dead: Vec::new(),
			resistant: Vec::new(),
			
			random: rand::thread_rng(),
			time_delta: 24,
			time_since_start: 0,
			infectivity,
			contact_no,
			incubation_period,
			fatality_rate,
			avg_hospitalisation_period,
			reinfection_rate,
			curabilty
		}
	}
}

impl Simulation {
	pub fn cycle(&mut self) {
		self.time_since_start += self.time_delta as usize;
	
		//Infections==========================================
		let mut i: usize = 0;
		'outer: 
		for p_infected in &self.infected {
			for _ in 0..self.contact_no {
				if i < self.susceptible.len() {
					let chance = self.random.gen::<f64>();
					if self.infectivity > chance {
						self.susceptible.swap_remove(i);				//Replace with swap remove to increase sim speed
						self.exposed.push(Person::Exposed(0));
					}
					else {
						i += 1;
					}
				}
				else {
					break 'outer;
				}
			}
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
				if ((*time_since_infected as f64) >= incubation_period) {
					self.infected.push(Person::Infectious(*time_since_infected,false));
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
		
		//If an infected has survived hospitaliztion period then turn them into resistant===
		let avg_hospitalisation_period = self.avg_hospitalisation_period;
		
		for (i,p_infected) in self.infected.iter_mut().enumerate() {
			if let Person::Infectious(ref mut time_since_infected,ref mut to_remove) = p_infected {
				*time_since_infected += self.time_delta;
				if ((*time_since_infected as f64) >= avg_hospitalisation_period) {
					let cure_chance = self.random.gen::<f64>();
					if self.curabilty >= cure_chance {
						*to_remove = true;
						let chance = self.random.gen::<f64>();
						if self.reinfection_rate >= chance {
							self.exposed.push(Person::Exposed(0));
						}
						else {
							self.resistant.push(Person::Resistant);
						}
					}
				}
			}	
		}
		
		self.infected.retain(|p_infected| {
			if let Person::Infectious(_,to_remove) = p_infected {
				return !to_remove;
			}
			return false;//Unreachable
		});
		
		//=====================================================================================
		
		//Check to see if they are dead=====================================================
		let init_infected = self.infected.len();
		let fatality_rate = self.fatality_rate;
		let mut rand = self.random.clone();
		
		self.infected.retain(|p_infected| {
				let chance = rand.gen::<f64>();
				!(fatality_rate >= chance)
			}
		);
		
		for _ in 0..(init_infected - self.infected.len()) {
			self.dead.push(Person::Dead);
		}
		//===================================================================================
	}
	
	pub fn stringify(&self) -> String {
		format!("Susceptible: {}            \nExposed: {}            \nInfected: {}            \nResistant: {}            \nDead: {}            \n\n",self.susceptible.len(),self.exposed.len(),self.infected.len(),self.resistant.len(),self.dead.len())
	}
	
	pub fn condensed_stringify(&self) -> String {
		format!("{} hrs:-\nSusceptible:{}\nExposed:{}\nInfected:{}\nResistant:{}\nDead:{}\n==============\n",self.time_since_start,self.susceptible.len(),self.exposed.len(),self.infected.len(),self.resistant.len(),self.dead.len())
	}
}

fn path_exists(path: &str) -> bool {
	std::fs::metadata(path).is_ok()
}

fn main() {
	println!("\x1B[2J");
	println!("\x1B[0;0H");
	
	let mut i = 1;
	let mut path = format!("./test{}.txt",i);
	while path_exists(&path) {
		i += 1;
		path = format!("./test{}.txt",i);
	}
	
	let f = File::create(&path).expect("Unable to create file");
	
	
	let mut writer = BufWriter::new(f);
	
	
	let mut sim = Simulation::from_file("./disease.txt");//Simulation::new(7089900,0.75,10,240.0,0.002,336.0,0.14,0.70);
	
	//return;
	
	let now = time::Instant::now();
	let mut days = 0;
	loop {
		sim.cycle();
		println!("{}",sim.stringify());
		writer.write_all(sim.condensed_stringify().as_bytes()).expect("Unable to write to log");
		//writer.flush();
		println!("\x1B[0;0H");
		
		if sim.infected.len() == 0 && sim.exposed.len() == 0 && sim.susceptible.len() == 0 {
			days = sim.time_since_start / 24;	
			writer.write_all((format!("\n\nDisease took {} days to end",sim.time_since_start / 24)).as_bytes());
			writer.flush();
			break;
		}
	}
	let time_taken = now.elapsed().as_secs();
	println!("Total time taken is {} seconds and each day took {} secs to process",time_taken,(time_taken as f64) / (days as f64));
}