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

    f, axis = plot.subplots(2, sharex=True)
    axis[0].grid()
    axis[0].plot(altitude)
    axis[0].plot(np.convolve(altitude, np.ones(8), mode="valid") / 8)
    #axis[0].plot(avg_altitude)
    axis[1].plot(pressure)
    #axis[1].plot(avg_pressure)

    plot.show()


main()
