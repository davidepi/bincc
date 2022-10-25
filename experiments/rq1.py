import os
import sys
import pandas as pd
import numpy as np
from tqdm import tqdm
import matplotlib.pyplot as plt
import seaborn as sns

def doplot(df, title, xlab, ylab, name):
    plt.clf()
    plt.grid(True, axis="both", linestyle="--", color="#b0b0b0")
    plt.style.use("seaborn-colorblind")
    sns.lineplot(data=df)
    plt.title(title)
    plt.xlabel(xlab)
    plt.ylabel(ylab)
    plt.tight_layout()
    plt.savefig(name, bbox_inches='tight')

archs = ['aarch64-gcc','amd64-gcc']
opts = ['o0','o2', 'os']
max_nodes = 501 # displayed on the plot!
iterations = len(archs)*len(opts)*(max_nodes-2)
ga_dict = {}
gba_dict = {}
gbb_dict = {}
with tqdm(total=iterations, ncols=80) as pb:
    for arch in archs:
        df = pd.read_csv(f"{arch}-funcs.csv")
        for opt in opts:
            thisconfig = df.loc[df["config"]==f"{arch}-{opt}"]
            column_ga = [float("NaN"), float("NaN")]
            column_gba = [float("NaN"), float("NaN")]
            column_gbb = [float("NaN"), float("NaN")]
            # some stats first
            notrivial = thisconfig.loc[(thisconfig["original"]>1)]
            avg_nodes_stat = notrivial["original"].mean()
            max_nodes_stat = notrivial["original"].max()
            print(f"{arch}-{opt} original avg nodes per function {avg_nodes_stat}")
            print(f"{arch}-{opt} original max nodes per function {max_nodes_stat}")
            for osize in range(2,max_nodes):
                notrivial_limit = notrivial.loc[(thisconfig["original"]<=osize)]
                total = len(notrivial_limit.index)
                if total > 0:
                    perfect = notrivial_limit.loc[notrivial_limit["reduced"]==1]
                    perfect_no = len(perfect.index)
                    column_ga.append(perfect_no/total)
                    original = notrivial_limit["original"].sum()
                    reduced = notrivial_limit["reduced"].sum()
                    column_gba.append((original-reduced)/original)
                    imperfect = notrivial_limit.loc[notrivial_limit["reduced"]!=1]
                    original_imperfect = imperfect["original"].sum()
                    reduced_imperfect = imperfect["reduced"].sum()
                    column_gbb.append((original_imperfect-reduced_imperfect)/original_imperfect)
                pb.update(1)
            ga_dict[f"{arch}-{opt}"] = column_ga
            gba_dict[f"{arch}-{opt}"] = column_gba
            gbb_dict[f"{arch}-{opt}"] = column_gbb
ga = pd.DataFrame(ga_dict)
gba = pd.DataFrame(gba_dict)
gbb = pd.DataFrame(gbb_dict)
doplot(ga, title="Perfect reconstructions",xlab="Input CFG size", ylab="Perfect reconstructions %", name="rq1_ga.pdf")
doplot(gba, title="Node reduction",xlab="Input CFG size", ylab="Reduction %", name="rq1_gba.pdf")
doplot(gbb, title="Node reduction (Failed only)",xlab="Input CFG size", ylab="Reduction %", name="rq1_gbb.pdf")
