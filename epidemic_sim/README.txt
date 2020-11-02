Epidemic Simulations:-

Features:
1.External Configuration
2.Seperation of people into age groups

Data to be logged:
1 No of suceptible, exposed, infectious, cured, removed per day
2 Basic reproduction number

Each cycle
Infected people have a chance to turn susceptible people to exposed
Exposed peoples's timers get increased
Check to see if exposed are now infected
If an infected has survived the hospitalization period turn them into resistant
Every delta time apply fatality rate and if true kill the infected

TODO:
Implement loading plague settings