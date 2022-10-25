import sys
import os
from tqdm import tqdm
import re
import r2pipe
import pandas as pd
from itertools import combinations

jointp = [
    {"md5_finish_ctx", "sha1_finish_ctx"},
    {"imaxtostr", "offtostr", "inttostr"},
    {"getgroup", "getuser"},
    {"saferead", "safewrite"},
    {"getgidbyname", "getuidbyname"},
    {"imaxtostr", "umaxtostr"},
    {"sha512_read_ctx", "sha224_read_ctx", "sha384_read_ctx", "sha256_read_ctx",
"sha512_finish_ctx", "sha224_finish_ctx", "sha384_finish_ctx", "sha256_finish_ctx"},
    {"output_crc", "output_bsd", "output_sysv"},
    {"quotearg_custom_mem", "quotearg_custom",
        "quotearg_n_custom_mem", "quotearg_n_custom"},
    {"quotearg_style", "quotearg_style_mem", "quotearg_n_style",
        "quotearg_n_style_colon", "quotearg_n_style_mem"},
    {"sha224_buffer", "sha256_buffer",
        "sha384_buffer", "sha512_buffer"},
    {"sha256_process_block", "sha512_process_block"},
    {"ftoastr", "dtoastr", "ldtoastr"},
    {"rev_xstrcoll_width", "rev_strcmp_width",
        "xstrcoll_width", "strcmp_width"},
    {"open_safer", "openat_safer", "opendir_safer"},
    {"xstrcoll_atime", "xstrcoll_ctime"},
    {"quotearg_char", "quotearg_char_mem"},
    {"xizalloc", "xzalloc"},
    {"xmemdup","ximemdup", "ximemdup0"},
    {"fd_safer", "fd_safer_flag"},
    {"AD_compare", "dev_ino_compare", "src_to_dest_compare"}
]


def compare(a, b, correct_set, wrong_set):
    if '.' in a:
        a = a[a.find('.')+1:]
    if '.' in b:
        b = b[b.find('.')+1:]
    if a > b:
        a, b = b, a
    if a == b and a != "main":
        correct_set.add((a,b))
    else:
        for subset in jointp:
            if a in subset and b in subset:
                correct_set.add((a,b))
                return
        wrong_set.add((a,b))

def deepbindiff(a, b, r2_a, r2_b, minsim):
    matches_filename = f"deepbindiff_{a}_{b}_matches.txt"
    indices_filename = f"deepbindiff_{a}_{b}_nodeIndexToCode.txt"

    matches = []
    with open(matches_filename, 'r') as fin:
        content = fin.read().replace('[','').replace(']','').replace(',','').split()
        content_no_empty = list(filter(None, content))
        content_int = [int(x) for x in content_no_empty]
        matches = list(zip(*(iter(content_int),) * 2))

    indices = {}
    with open(indices_filename, 'r') as fin:
        lines = fin.readlines()[1:]
        lines_no_lf = [x.strip() for x in lines if x!='\n']
        lines_zipped = list(zip(*(iter(lines_no_lf),)*2))
        for bb_id, content in lines_zipped:
            bb_id_int = int(bb_id[:-1])
            bb_offset_int = int(re.search("0x[0-9a-f]+", content).group(), 0)
            bb_offset_int_no_pic = bb_offset_int - 0x400000
            indices[bb_id_int] = bb_offset_int_no_pic

    function_pairs = {}
    for a, b in matches:
        a_offset = indices[a]
        b_offset = indices[b]
        r2_a.cmd(f's {a_offset}')
        r2_b.cmd(f's {b_offset}')
        bb_a_no = int(r2_a.cmd('afb | wc -l'))
        bb_b_no = int(r2_b.cmd('afb | wc -l'))
        if bb_a_no > 1 and bb_b_no > 1:
            a_offset = int(r2_a.cmdj('afij')[0]['offset'])
            b_offset = int(r2_b.cmdj('afij')[0]['offset'])
            if (a_offset, b_offset) in function_pairs:
                function_pairs[(a_offset, b_offset)] += 1
            else:
                function_pairs[(a_offset, b_offset)] = 1

    a_functions = r2_a.cmdj('aflj')
    b_functions = r2_b.cmdj('aflj')
    a_functions_map = {}
    b_functions_map = {}
    for function in a_functions:
        if function["nbbs"] > 1:
            a_functions_map[int(function['offset'])] = function
    for function in b_functions:
        if function["nbbs"] > 1:
            b_functions_map[int(function['offset'])] = function

    clones = set()
    for (offset_a, offset_b), count in function_pairs.items():
        bbs_a = int(a_functions_map[offset_a]["nbbs"])
        bbs_b = int(b_functions_map[offset_b]["nbbs"])
        if count >= bbs_a*minsim:
            name_a = a_functions_map[offset_a]["name"]
            name_b = b_functions_map[offset_b]["name"]
            clones.add((name_a, name_b))

    correct = set()
    wrong = set()
    for clone_a, clone_b in clones:
        compare(clone_a, clone_b, correct, wrong)
    return (correct, wrong)


##### BINDIFF ####
def bindiff(a, b, minsim):
    bd_res = pd.read_csv(f"bindiff_{a}_{b}.csv")
    bd_res = bd_res.loc[bd_res["bbs"]>1]
    bd_res = bd_res.loc[bd_res["similarity"]>minsim]
    correct = set()
    wrong = set()
    for _, row in bd_res.iterrows():
        clone_a = row["primary_name"]
        clone_b = row["secondary_name"]
        compare(clone_a, clone_b, correct, wrong)
    return (correct, wrong)

programs = ['dir','ls','mv','cp','sort','du','csplit','expr','nl','ptx','split']
minsim = 0.5
amd64_folder = os.getenv('AMD64')
r2du = r2pipe.open(f"{amd64_folder}/du", flags=['-2'])
r2du.cmd('aaa')
bd_prec_sum = 0
dbd_prec_sum = 0
for b in programs:
    r2_b = r2pipe.open(f"{amd64_folder}/{b}", flags=['-2'])
    r2_b.cmd('aaa')
    dbd_correct, dbd_wrong = deepbindiff('du', b, r2du, r2_b, minsim)
    r2_b.quit()
    bd_correct, bd_wrong = bindiff('du', b, minsim)
    dbd_tp, dbd_fp = len(dbd_correct), len(dbd_wrong)
    bd_tp, bd_fp = len(bd_correct), len(bd_wrong)
    dbd_precision =  dbd_tp/(dbd_tp+dbd_fp)
    bd_precision = bd_tp/(bd_tp+bd_fp)
    bd_prec_sum += bd_precision
    dbd_prec_sum += dbd_precision
    print(f"{b} & {bd_precision:.4f} ({bd_fp+bd_tp}) & {dbd_precision:.4f} ({dbd_fp+dbd_tp})")
r2du.quit()
print("---------------------")
print(f"mean & {bd_prec_sum/len(programs):.4f} & {dbd_prec_sum/len(programs):.4f}")
