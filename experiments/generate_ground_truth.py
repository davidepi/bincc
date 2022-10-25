#!python3
import pandas as pd
from tqdm import tqdm
import csv
from collections import defaultdict

jointp = [
    {"dbg.md5_finish_ctx", "dbg.sha1_finish_ctx"},
    {"dbg.imaxtostr", "dbg.offtostr", "dbg.inttostr"},
    {"dbg.getgroup", "dbg.getuser"},
    {"dbg.saferead", "dbg.safewrite"},
    {"dbg.getgidbyname", "dbg.getuidbyname"},
    {"dbg.imaxtostr", "dbg.umaxtostr"},
    {"sym.sha512_read_ctx", "sym.sha224_read_ctx", "sym.sha384_read_ctx", "sym.sha256_read_ctx",
"sym.sha512_finish_ctx", "sym.sha224_finish_ctx", "sym.sha384_finish_ctx", "sym.sha256_finish_ctx"},
    {"dbg.output_crc", "dbg.output_bsd", "dbg.output_sysv"},
    {"sym.quotearg_custom_mem", "sym.quotearg_custom",
        "sym.quotearg_n_custom_mem", "sym.quotearg_n_custom"},
    {"sym.quotearg_style", "sym.quotearg_style_mem", "sym.quotearg_n_style",
        "sym.quotearg_n_style_colon", "sym.quotearg_n_style_mem"},
    {"dbg.sha224_buffer", "dbg.sha256_buffer",
        "dbg.sha384_buffer", "dbg.sha512_buffer"},
    {"dbg.sha256_process_block", "dbg.sha512_process_block"},
    {"dbg.ftoastr", "dbg.dtoastr", "dbg.ldtoastr"},
    {"dbg.rev_xstrcoll_width", "dbg.rev_strcmp_width",
        "dbg.xstrcoll_width", "dbg.strcmp_width"},
    {"dbg.open_safer", "dbg.openat_safer", "dbg.opendir_safer"},
    {"dbg.xstrcoll_atime", "dbg.xstrcoll_ctime"},
    {"sym.quotearg_char", "sym.quotearg_char_mem"},
    {"dbg.xizalloc", "dbg.xzalloc"},
    {"sym.xmemdup","dbg.ximemdup", "dbg.ximemdup0"},
    {"dbg.fd_safer", "dbg.fd_safer_flag"},
    {"dbg.AD_compare", "dbg.dev_ino_compare", "dbg.src_to_dest_compare"}
]


def clone_map(arch):
    df = pd.read_csv(f"function_names.csv")
    funcs = defaultdict(set)
    config = df.loc[df["arch"] == arch]
    print(f"reading {arch} functions list")
    with tqdm(total=len(config), ncols=80) as pb:
        for _, row in config.iterrows():
            bin = row["bin"]
            func = row["func"]
            funcs[func].add(bin)
            pb.update(1)
    # these are not clones
    funcs.pop("main", None)
    funcs.pop("dbg.main", None)
    funcs.pop("entry.fini0", None)
    funcs.pop("entry.init0", None)
    return funcs, config


func_done = set()
funcs_amd64, config_amd64 = clone_map('amd64')
funcs_aarch64, config_aarch64 = clone_map('aarch64')
clone_class_id = -1
with open(f"ground_truth_same.csv", "w", newline="") as csvfile_same:
    with open(f"ground_truth_cross.csv", "w", newline="") as csvfile_cross:
        writer_same = csv.writer(csvfile_same, delimiter=",", quotechar="\"", quoting=csv.QUOTE_MINIMAL)
        writer_same.writerow(["arch", "bits", "binary", "function", "clone_class_id", "depth"])
        writer_cross = csv.writer(csvfile_cross, delimiter=",", quotechar="\"", quoting=csv.QUOTE_MINIMAL)
        writer_cross.writerow(["arch", "bits", "binary", "function", "clone_class_id", "depth"])
        for (func, bin) in funcs_amd64.items():
            func_set = {func}
            # check if the current function has to be joined with other similar ones
            for i, func_tp_set in enumerate(jointp):
                if func in func_tp_set:
                    func_set = func_tp_set
            clone_class_id += 1
            if func not in func_done:
                for func in func_set:
                    func_done.add(func)
                    assert func in funcs_amd64
                    bin_set = funcs_amd64[func]
                    single_bin = False
                    this_func_amd64 = config_amd64.loc[config_amd64["func"] == func]
                    if len(bin_set) > 1:
                        assert len(this_func_amd64) > 0
                        for bin in bin_set:
                            this_bin = this_func_amd64.loc[this_func_amd64["bin"] == bin]
                            depth = this_bin.iloc[0]["depth"]
                            writer_same.writerow(["x86", "64", bin, func, clone_class_id, depth])
                            writer_cross.writerow(["x86", "64", bin, func, clone_class_id, depth])
                    elif len(bin_set) == 1:
                        this_bin = this_func_amd64.loc[this_func_amd64["bin"] == next(
                            iter(bin))]
                        if not this_bin.empty:
                            single_bin = True

                    # now add the aarch64 counterparts
                    if func in funcs_aarch64:
                        bin_set_aarch = funcs_aarch64[func]
                        if len(bin_set) > 0:
                            added_aarch64_bin = 0
                            this_func_aarch64 = config_aarch64.loc[config_aarch64["func"] == func]
                            assert len(this_func_aarch64) > 0
                            for bin in bin_set:
                                this_bin = this_func_aarch64.loc[this_func_aarch64["bin"] == bin]
                                if not this_bin.empty:
                                    depth = this_bin.iloc[0]["depth"]
                                    writer_cross.writerow(["arm", "64", bin, func, clone_class_id, depth])
                                    added_aarch64_bin += 1
                            # add the single function from amd64 that was excluded before
                            if single_bin and added_aarch64_bin > 0:
                                this_bin = this_func_amd64.loc[this_func_amd64["bin"] == next(iter(bin))]
                                if not this_bin.empty:
                                    this_bin = this_func_amd64.loc[this_func_amd64["bin"] == next(iter(bin))]
                                    depth = this_bin.iloc[0]["depth"]
                                    writer_cross.writerow(["x86", "64", bin, func, clone_class_id, depth])
