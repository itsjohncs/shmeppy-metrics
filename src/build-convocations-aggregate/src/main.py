#!/usr/bin/env python3

import datetime
import json
import os
import subprocess
import sys
import multiprocessing.dummy

import tqdm


SCRIPT_DIR = os.path.dirname(os.path.realpath(__file__))


def get_date_range_of_new_logs(last_seen_sizes_path, raw_logs_dir):
    """Return range of dates with new log entires.

    It's possible for there to be sparse log entries such that (for example)
    there's logs for 2020-01-01, 2020-01-03, and 2020-01-04. It'd be more
    optimal if we returned two date ranges in this case, but we don't. We'd
    return the range 2020-01-01→2020-01-04.

    master_file_path won't be updated.
    """
    try:
        with open(last_seen_sizes_path, "rb") as f:
            last_seen_sizes = json.load(f)
    except FileNotFoundError as e:
        last_seen_sizes = {}

    raw_log_paths = [
        os.path.join(raw_logs_dir, name)
        for name in os.listdir(raw_logs_dir)
        if name.endswith(".log")
    ]

    # Check that no log files have been deleted
    deleted_log_files = set(last_seen_sizes.keys()) - set(raw_log_paths)
    if deleted_log_files:
        raise AssertionError(
            f"Log files have been deleted since last run: {deleted_log_files}")

    children = []
    for path in raw_log_paths:
        last_seen_size = last_seen_sizes.get(path, 0)

        # (Roughly) check that no log entries have been deleted. It still
        # could've been modified that some log entries were lost but the total
        # file size grew. We have no way to detect this case, but it would
        # cause an inconsistent cache 🙃🤞.
        current_size = os.stat(path).st_size
        if current_size < last_seen_size:
            raise AssertionError(
                f"{path} has length {current_size} but expected "
                f"{last_seen_size} or greater.")

        # Parallel execution is likely to be beneficial here (despite the
        # heavily IO bound nature of the programs) because of the heavy file
        # caching that's likely occurring (and possible mmap usage, though that
        # hasn't been added to the fast-log-utils as of this comment's
        # writing).
        children.append(subprocess.Popen(
            [
                os.path.join(SCRIPT_DIR, "skip-bytes-and-get-range.sh"),
                str(last_seen_size),
                path,
            ],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True))

    start_min = None
    end_max = None
    for p in children:
        stdout, stderr = p.communicate(timeout=30)
        if p.returncode != 0 or stderr.strip() != "":
            raise RuntimeError(
                f"Child exited with status {p.returncode}\n\n{stderr}")

        # The script will return 0 and print nothing if there's no new log
        # entries.
        if stdout.strip() == "":
            continue

        raw_start, raw_end = stdout.split()

        start = datetime.date.fromisoformat(raw_start)
        if start_min is None or start < start_min:
            start_min = start

        end = datetime.date.fromisoformat(raw_end)
        if end_max is None or end > end_max:
            end_max = end

    return start_min, end_max


def refresh_last_seen_sizes(cache_dir, raw_logs_dir):
    raw_log_paths = [
        os.path.join(raw_logs_dir, name)
        for name in os.listdir(raw_logs_dir)
        if name.endswith(".log")
    ]
    last_seen_sizes = {path: os.stat(path).st_size for path in raw_log_paths}
    with open(os.path.join(cache_dir, "last-seen-sizes.json"), "w") as f:
        json.dump(last_seen_sizes, f)


def get_dates_to_build(cache_dir, raw_logs_dir):
    dates_to_build = set()

    # This is an inclusive range of dates that we have new logs for
    start, end_inclusive = get_date_range_of_new_logs(
        os.path.join(cache_dir, "last-seen-sizes.json"),
        raw_logs_dir)

    # We don't have any new log entries since the last rebuild
    if start is None and end_inclusive is None:
        return dates_to_build

    # Get all the dates to rebuild
    for name in os.listdir(cache_dir):
        if name == "last-seen-sizes.json" or not name.endswith(".json"):
            continue

        date = datetime.date.fromisoformat(name[:-len(".json")])
        cache_keys = [
            date - datetime.timedelta(days=1),
            date,
            date + datetime.timedelta(days=1),
        ]
        if any(start <= k <= end_inclusive for k in cache_keys):
            dates_to_build.add(date)

    # Get all the dates in the range of new log entries we have
    i = start
    while i <= end_inclusive:
        dates_to_build.add(i)
        i += datetime.timedelta(days=1)

    return dates_to_build


def build_cache_for_day(date, cache_dir, raw_logs_dir):
    subprocess.run(
        [
            os.path.join(SCRIPT_DIR, "build-cache-for-day.sh"),
            raw_logs_dir,
            os.path.join(cache_dir, f"{date.isoformat()}.json"),  # output file
            date.isoformat(),  # day
            (date + datetime.timedelta(days=1)).isoformat(),  # surrounding_day
            (date - datetime.timedelta(days=1)).isoformat(),  # surrounding_day
        ], check=True, timeout=60*5)


def maybe_update_cache(cache_dir, raw_logs_dir):
    dates_to_build = get_dates_to_build(cache_dir, raw_logs_dir)

    if dates_to_build:
        pool = multiprocessing.dummy.Pool()
        for _ in tqdm.tqdm(
                pool.imap_unordered(
                    lambda d: build_cache_for_day(d, cache_dir, raw_logs_dir),
                    dates_to_build),
                total=len(dates_to_build),
                desc="updating cache"):
            pass


def create_aggregate(aggregate_path, cache_dir):
    cache_paths = [(name, os.path.join(cache_dir, name))
                   for name in os.listdir(cache_dir)
                   if name != "last-seen-sizes.json" and
                      name.endswith(".json")]

    aggregate = {}
    for name, path in cache_paths:
        with open(path, "r") as f:
            aggregate[name[:-len(".json")]] = json.load(f)

    with open(aggregate_path, "w") as f:
        json.dump(aggregate, f)

def main(aggregate_path, cache_dir, raw_logs_dir):
    maybe_update_cache(cache_dir, raw_logs_dir)
    refresh_last_seen_sizes(cache_dir, raw_logs_dir)
    create_aggregate(aggregate_path, cache_dir)



if __name__ == "__main__":
    main(*sys.argv[1:])
