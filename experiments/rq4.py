import os
import sys

import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns
from sklearn.ensemble import IsolationForest
import numpy as np

def remove_outliers(df):
    values = df[["size","time"]].to_numpy()
    clf = IsolationForest(contamination=0.01).fit(values)
    prediction = np.array(clf.predict(values) > 0)
    df = df[prediction]
    return df

def preprocess(df):
    disasm = df.drop(["cfs"], axis=1)
    cfs = df.drop(["disasm"], axis=1)
    disasm["type"] = "Disassembly"
    cfs["type"] = "Structural Analysis"
    disasm = disasm.rename(columns={"disasm": "time"})
    cfs = cfs.rename(columns={"cfs": "time"})
    concat = pd.concat([disasm,cfs])
    return (disasm, cfs)

def stacked():
    df = pd.read_csv("amd64-gcc-detailed-times.csv")
    df = df.loc[(df != 0).all(axis=1), :]
    df['Semantic Analysis'] = df.loc[:,['cfs_time','fvec_time','combined_time']].sum(axis=1)/1000000
    #df['Total'] = df.loc[:,['disasm_time','cfs_time','fvec_time','combined_time']].sum(axis=1)/1000000
    df = df.rename(columns={"disasm_time":"Disassembly","cfs_time":"Structural Analysis"})
    df['Disassembly'] = df['Disassembly']/1000000
    df['Structural Analysis'] = df['Structural Analysis']/1000000
    df['total_size'] = df['total_size']/(1024*1024)
    df =  df.sort_values(by=["total_size"])
    df = df.drop(["fvec_time","combined_time","structural_time","bin"], axis=1)
    plt.clf()
    plt.grid(True, axis="both", linestyle="--", color="#b0b0b0", zorder=0)
    plt.style.use("seaborn-v0_8-colorblind")
    plt.stackplot(df["total_size"].to_numpy(), df["Structural Analysis"].to_numpy(), df["Semantic Analysis"].to_numpy(), df["Disassembly"].to_numpy(), zorder=3, labels=["Structural Analysis","Semantic Analysis","Disassembly"])
    plt.title("Composition of time required for the combined analysis")
    plt.legend(loc="upper left")
    plt.xlabel("Input size (MiB)")
    plt.ylabel("Time(s)")
    plt.tight_layout()
    plt.savefig("rq4c.pdf", bbox_inches='tight')

def doplot(df, title, xlab, ylab, name):
    plt.clf()
    plt.grid(True, axis="both", linestyle="--", color="#b0b0b0")
    plt.style.use("seaborn-colorblind")
    sns.scatterplot(data=df, x="size", y="time", markers=["+", "x"], hue="type")
    plt.title(title)
    plt.xlabel(xlab)
    plt.ylabel(ylab)
    plt.tight_layout()
    plt.savefig(name, bbox_inches='tight')


df = None
df = pd.read_csv("amd64-gcc-times.csv")
df = df.loc[df["config"] != "amd64-gcc-o2"]
df = df.drop(["config", "exec"], axis=1)
(disasm, cfs) = preprocess(df)
disasm = remove_outliers(disasm)
cfs = remove_outliers(cfs)
df0 = cfs
df1 = pd.concat([cfs, disasm])

# only cfs
df0["size"] = df0["size"]/(1024*1024)
df0["time"] = df0["time"]/1000;
doplot(df0, title="Time required for the analysis",xlab="Input size (MiB)", ylab="Time (s)", name="rq4b.pdf")
# disasm + cfs
df1["size"] = df1["size"]/(1024*1024)
df1["time"] = df1["time"]/1000;
doplot(df1, title="Time required for the analysis",xlab="Input size (MiB)", ylab="Time (s)", name="rq4a.pdf")
# stacked
stacked()

