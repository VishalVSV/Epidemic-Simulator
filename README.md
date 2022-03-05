# Epidemic-Simulator
A super simple, super inaccurate epidemic simulator loosely based on the SEIR model.

This simulator takes in a file with settings for the simulator and applies a set of rules to it till there are no susceptible, exposed or infected people in the population.

In this model the susceptible on exposure to an infected person become exposed. 
After an incubation period they become infected and then have a change to receive treatment after which they have a chance to be cured.
After which they have a chance to become resistant to the disease.

## Images
These are comparsions between mostly made up values designed to closely match covid 19 and the spanish flu.

### Daily deaths 
![Deaths](https://raw.githubusercontent.com/VishalVSV/Epidemic-Simulator/master/images/Comparision/Daily_Deaths.png)

### Daily Infections 
![Infections](https://raw.githubusercontent.com/VishalVSV/Epidemic-Simulator/master/images/Comparision/Daily_Infections.png)

### Number of people dead over time 
![Deaths Over Time](https://raw.githubusercontent.com/VishalVSV/Epidemic-Simulator/master/images/Comparision/Dead.png)

### Number of people exposed over time 
![exposed](https://raw.githubusercontent.com/VishalVSV/Epidemic-Simulator/master/images/Comparision/Exposed.png)

### Number of people infected over time 
![Deaths](https://raw.githubusercontent.com/VishalVSV/Epidemic-Simulator/master/images/Comparision/Infections.png)

### Number of people resistant over time
![Deaths](https://raw.githubusercontent.com/VishalVSV/Epidemic-Simulator/master/images/Comparision/Resistant.png)

### Number of people susceptible over time
![Deaths](https://raw.githubusercontent.com/VishalVSV/Epidemic-Simulator/master/images/Comparision/Susceptible.png)

Written in Rust and graphed in C#.
