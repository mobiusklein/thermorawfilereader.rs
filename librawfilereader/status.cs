using System;

using System.Collections.Generic;

namespace librawfilereader
{
    public class StatusLog<T> {
        public string Name;
        public List<T> Data;
        public List<double> Time;

        public StatusLog(string name) {
            Name = name;
            Data = new List<T>();
            Time = new List<double>();
        }

        public StatusLog(string name, List<T> data, List<double> time) {
            Name = name;
            Data = data;
            Time = time;
        }

        public void Add(T datum, double time) {
            Data.Add(datum);
            Time.Add(time);
        }
    }
}