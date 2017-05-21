#!/bin/python

import numpy as np
import scipy
import matplotlib
import matplotlib.pyplot as plot

import sys

import json



def main():
    if len(sys.argv) != 2:
        print("Please specify an input filename")
        return

    filename = sys.argv[1]

    data = None
    
    with open(filename) as f:
        file_content = f.read()
        data = json.loads(file_content)

    # Create separate numpy array for all the data
    altitude = np.empty(len(data))
    avg_altitude = np.empty(len(data))
    pressure = np.empty(len(data))
    avg_pressure = np.empty(len(data))

    for i in range(len(data)):
        print("Altitude: ", data[i]['alt'], "average: ", data[i]["avg_alt"])
        altitude[i] = data[i]["alt"]
        avg_altitude[i] = data[i]["avg_alt"]
        pressure[i] = data[i]["p"]
        avg_pressure[i] = data[i]["avg_p"]

    smoothed = np.convolve(altitude, np.ones(16), mode="valid") / 16

    velocity = np.empty(len(smoothed))

    for i in range(1, len(velocity)):
        velocity[i] = smoothed[i]-smoothed[i-1]

    f, axis = plot.subplots(2, sharex=True)
    axis[0].grid()
    axis[0].plot(altitude)
    axis[0].plot(smoothed)
    #axis[0].plot(avg_altitude)
    axis[1].plot(velocity)
    axis[1].plot(np.convolve(velocity, np.ones(16), mode='valid') / 16)
    #axis[1].plot(avg_pressure)

    plot.show()


main()
