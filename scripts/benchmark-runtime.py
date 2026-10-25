#!/usr/bin/env python3
"""Runs a built game binary several times for a fixed time, standing still in the showcase room,
and prints one Markdown table row with the averages.

Usage: benchmark-runtime.py BINARY LABEL [--seconds N] [--runs N] [-- extra game arguments]

Measures: time to first frame and frame times (the game's own report), CPU use (user + system
time over wall time), peak resident memory, GPU memory (amdgpu `drm-memory-vram` in
/proc/PID/fdinfo, sampled halfway through) and thread count (sampled halfway through).
"""

import argparse
import os
import re
import subprocess
import time

REPORT_PATTERN = re.compile(
    r"first frame after (?P<first>[\d.]+) ms; \d+ frames, mean (?P<mean>[\d.]+) ms \((?P<fps>\d+) fps\), "
    r"median [\d.]+ ms, 99th percentile (?P<p99>[\d.]+) ms, slowest (?P<slowest>[\d.]+) ms"
)
VRAM_PATTERN = re.compile(r"^drm-memory-vram:\s+(\d+)\s+KiB", re.MULTILINE)
CLIENT_PATTERN = re.compile(r"^drm-client-id:\s+(\d+)", re.MULTILINE)
KIBIBYTES_PER_MEBIBYTE = 1024


def gpu_memory_mebibytes(process_id):
    """Sums VRAM over the process's DRM clients (several descriptors can share one client)."""
    vram_by_client = {}
    fdinfo_directory = f"/proc/{process_id}/fdinfo"
    for descriptor in os.listdir(fdinfo_directory):
        try:
            with open(os.path.join(fdinfo_directory, descriptor)) as fdinfo:
                text = fdinfo.read()
        except OSError:
            continue
        client = CLIENT_PATTERN.search(text)
        vram = VRAM_PATTERN.search(text)
        if client and vram:
            vram_by_client[client.group(1)] = int(vram.group(1))
    return sum(vram_by_client.values()) / KIBIBYTES_PER_MEBIBYTE


def thread_count(process_id):
    with open(f"/proc/{process_id}/status") as status:
        return int(re.search(r"^Threads:\s+(\d+)", status.read(), re.MULTILINE).group(1))


def run_once(binary, seconds, extra_arguments):
    started = time.monotonic()
    process = subprocess.Popen(
        [binary, "--exit-after", str(seconds), *extra_arguments],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    time.sleep(seconds / 2)
    gpu_memory = gpu_memory_mebibytes(process.pid)
    threads = thread_count(process.pid)
    output = process.stdout.read()
    _, _, usage = os.wait4(process.pid, 0)
    process.returncode = 0
    wall_seconds = time.monotonic() - started
    report = REPORT_PATTERN.search(output)
    if report is None:
        raise SystemExit(f"no frame report in output:\n{output}")
    return {
        "first": float(report["first"]),
        "mean": float(report["mean"]),
        "fps": float(report["fps"]),
        "p99": float(report["p99"]),
        "slowest": float(report["slowest"]),
        "cpu": 100 * (usage.ru_utime + usage.ru_stime) / wall_seconds,
        "rss": usage.ru_maxrss / KIBIBYTES_PER_MEBIBYTE,
        "vram": gpu_memory,
        "threads": threads,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("binary")
    parser.add_argument("label")
    parser.add_argument("--seconds", type=float, default=10.0)
    parser.add_argument("--runs", type=int, default=3)
    parser.add_argument("extra", nargs="*", help="arguments passed to the game (after --)")
    arguments = parser.parse_args()
    runs = [run_once(arguments.binary, arguments.seconds, arguments.extra) for _ in range(arguments.runs)]
    average = {key: sum(run[key] for run in runs) / len(runs) for key in runs[0]}
    print(
        f"| {arguments.label} | {average['first']:.0f} ms | {average['mean']:.2f} ms ({average['fps']:.0f} fps) "
        f"| {average['p99']:.2f} ms | {average['slowest']:.1f} ms | {average['cpu']:.1f} % "
        f"| {average['rss']:.0f} MiB | {average['vram']:.0f} MiB | {average['threads']:.0f} |"
    )


if __name__ == "__main__":
    main()
