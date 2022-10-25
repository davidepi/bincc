#!python3
import os
import sys
import pandas as pd
from tqdm import tqdm

# from a dataframe to clone classes (list of sets)


def to_ccs(df):
    map = list()
    total = df["clone_class_id"].max()+1
    for i in range(0, total):
        classdf = df.loc[df["clone_class_id"] == i]
        clones = set()
        for _, row in classdf.iterrows():
            arch = row["arch"]
            bits = row["bits"]
            bin = os.path.basename(row["binary"])
            func = row["function"]
            clones.add("[" + arch + str(bits) + "]" + bin + "::" + func)
        map.append(clones)
    return map


ground_truth_path = sys.argv[1]
results_path = sys.argv[2]
min_depth = int(sys.argv[3])
ground_truth = pd.read_csv(ground_truth_path)
results = pd.read_csv(results_path)
clone_classes_no = results["clone_class_id"].max()
unreduced = ground_truth.loc[ground_truth["depth"] == 0]
filtered_truth = ground_truth.loc[ground_truth["depth"] >= min_depth]
filtered_results = results.loc[results["class_depth"] >= min_depth]
gt = to_ccs(filtered_truth)
res = to_ccs(filtered_results)
tp, fp, fn = 0, 0, 0
for cc in tqdm(res):
    subset = False
    for set2 in res:
        if cc != set2 and cc.issubset(set2):
            subset = subset or True
    if subset:
        continue
    best_score, best_other = 0.0, set()
    for i, other in enumerate(gt):
        union = cc.union(other)
        intersection = cc.intersection(other)
        score = len(intersection)/len(union)
        if score > best_score:
            best_score, best_other = score, other
    tp += len(cc.intersection(best_other))
    fp += len(cc.difference(best_other))
    fn += len(best_other.difference(cc))
for cc in tqdm(gt):
    subset = False
    for set2 in gt:
        if cc != set2 and cc.issubset(set2):
            subset = subset or True
    if subset:
        continue
    best_score, best_other = 0.0, set()
    for i, other in enumerate(res):
        score = len(cc.intersection(other))
        if score > best_score:
            best_score, best_other = score, other
    if best_score == 0.0:
        fn += len(cc)
precision = tp/(tp+fp)
recall = tp/(tp+fn)
fscore = 2*tp/(2*tp+fp+fn)
print(f"FSCORE:{fscore:.4f} => {tp} & {fp} & {fn} & {precision:.4f} & {recall:.4f}")
