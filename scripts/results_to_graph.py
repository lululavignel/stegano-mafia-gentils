import matplotlib.pyplot as plt
import numpy as np

from pathlib import Path
from os.path import join
import os
working_dir=f"{str(Path.home())}/Images/steg"


class data_meth:
    def __init__(self) -> None:
        self.methods={"SE0xFF":{},"SE0x03":{},"SER0x02":{}}
        self.steganograph={"s256-c-p": self.methods.copy()}

    def add_data(self,data, method, steg, percentage):
        if percentage in self.steganograph[steg][method]:
            self.steganograph[steg][method][percentage].append(data)
        else:
            self.steganograph[steg][method][percentage]=[data]
    def get_data(self,steg,method):
        return self.steganograph[steg][method]
    

methods={"SE0xFF":[{}],"SE0x03":[{}],"SER0x02":[{}]}
steganograph={"s256-c-p": [methods.copy()]}
steg_meths=["s256-c-p"]
data_aggregator=data_meth()

for top_dirnames in os.listdir(working_dir):
    cur_dos=join(working_dir,top_dirnames)
    if  os.path.isfile(cur_dos):
        continue

    for filename in  os.listdir(cur_dos):
        cur_file = join(cur_dos,filename)
        if os.path.isdir(cur_file) and not cur_file.endswith("png"):
            continue
        cur_per=top_dirnames
        if "base_img" in top_dirnames:
            for method in methods.keys():
                file = open(cur_dos+"/logs/"+method)
                data = file.readlines()
                file.close()
                for line in data:
                    value = float(line.split(";")[-1].strip())
                    for steg in steg_meths:
                        print("aaaaaaa   ",value,method,steg)
                        data_aggregator.add_data(value,method,steg,0)

        else :
            method=""
            for cur_method in steg_meths:
                if cur_method in top_dirnames:
                    method=cur_method
            if method=="":
                continue
            print(methods.keys())
            for method in methods.keys():
                file = open(cur_dos+"/logs/"+method)
                data = file.readlines()
                file.close()
                for line in data:
                    value = float(line.split(";")[-1].strip())
                    for steg in steg_meths:
                        data_aggregator.add_data(value,method,steg,float(cur_dos.split(".")[-1].strip()))



                
for method in data_aggregator.methods.keys():

    datas = data_aggregator.get_data("s256-c-p",method)
    keys = datas.keys()
    printable_data =[]
    print (keys)
    ordered_keys=[]
    for key in keys:
        ordered_keys.append(key)
    ordered_keys.sort()
    for key in ordered_keys:
        printable_data.append(datas[key])
    print(ordered_keys)
    for i in range(len(ordered_keys)):
        a = ordered_keys[i]
        while a> 0.99999:
            a/=10
        ordered_keys[i]=a*100
    plt.boxplot(printable_data,labels=ordered_keys)
    #plt.yscale("log")
    plt.xlabel("Pourcentage de la capacité max de stéganographie")
    plt.ylabel("Score du test de stéganalyse")
    plt.title(f"Résultats de la stéganalyse par l'algorithme {method}")
    
    # show plot
    plt.show()