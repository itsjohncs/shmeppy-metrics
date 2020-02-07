#!/usr/bin/env python3

import datetime
import json
import os
import subprocess
import sys


SCRIPT_DIR = os.path.dirname(os.path.realpath(__file__))


def get_date_range_of_new_logs(master_file_path, raw_logs_dir):
    """Return range of dates with new log entires.

    It's possible for there to be sparse log entries such that (for example)
    there's logs for 2020-01-01, 2020-01-03, and 2020-01-04. It'd be more
    optimal if we returned two date ranges in this case, but we don't. We'd
    return the range 2020-01-01→2020-01-04.

    master_file_path won't be updated.
    """
    try:
        with open(master_file_path, "rb") as f:
            last_seen_sizes = json.load(f)
    except FileNotFoundError as e:
        last_seen_sizes = {}

    raw_log_paths = [
        os.path.join(raw_logs_dir, name)
        for name in os.listdir(raw_logs_dir)
        if name.endswith(".log")
    ]

    # Check that no log files have been deleted
    deleted_log_files = set(last_seen_sizes.keys()) - set(raw_logs_dir)
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

get_date_range_of_new_logs(
    "/tmp/noexist",
    "/Users/johnsullivan/personal/shmeppy-admin/shmeppy-metrics/raw-logs/")

sys.exit()




def date_to_string(date):
    return date.strftime("%Y-%m-%d")


def get_cache_keys(date):
    return [
        date - datetime.timedelta(days=1),
        date,
        date + datetime.timedelta(days=1),
    ]

def date_in_range(date, start, end):
    return start <= date <= end


def build_cache():
    start_inclusive, end_inclusive = (
        get_date_range_of_new_logs(master_file_path, raw_logs_dir))

    dates_to_build = set()

    # Get all the dates to rebuild
    for date in all_dates_in(cache_dir):
        if any(start_inclusive <= d <= end_inclusive
               for d in get_cache_keys(date_from_path(file_path))):
            dates_to_rebuild.add(date)

    # Get all the dates in the range
    dates_to_build.extend(dates_in(start_inclusive, end_inclusive))

    build_em()



def should_build_cache_entry(date, raw_logs_dir, cache_dir):
    """

    Every time the cache is updated, we note the names of the log files that we
    used to update it, and the sizes of those log files. Because log files are
    assumed to be unique, append-only, and never deleted, this lets us
    determine the date range we have new log entries for easily.

    Each cache entry has a date as its filename. A cache entry is
    invalidated every time we get a new log entry with a timestamp for that
    date, or the days surrounding it. So each cache entry kind of has three
    "keys": its date from its filename, the day before it, and the day after
    it.

    The process goes: (1) determine the range of dates we have new log entries
    for, (2) rebuild any existing log entries that has a key in that range,
    (3) build cache entries of all dates in that range (notice that the date
    range of entries we rebuild can be wider than the date range of entries
    we'd build), (4) "atomically" (to the best of my ability, mac is shitty at
    atomic FS operations) update the master file containing the names of log
    files and their sizes.

    This process allows the cache to become inconsistent if it fails partway
    through. But in that case, the master file's file sizes (or set of file
    names) will not match what's on the file system. So to ensure we never read
    inconsistent data, we just have to be careful to never read from the cache
    when the master file is out of date.
    """

    # Assumptions:
    #  * Raw logs are append-only
    #  * Raw logs are ordered such that log message X has no log message Y such
    #    that all of the following are true: (1) Y appears in the same file as
    #    X, (2) Y appears before X in that file, and (3) Y's timestamp is
    #    strictly more than 24 hours before X's timestamp (careful here, 24
    #    hours is not necessarily 1 day because time sucks, though that
    #    consideration may not come into play in practice since this assumption
    #    is wishy-washy anyways and not actually built on any guarentees of the
    #    application).
    # I'm tempted to peek the first and last day in each raw log and use that
    # as inputs as opposed to each raw log's modification timestamp. How would
    # I use those inputs though?


def cache_convocations_for_day(date, build_dir, raw_logs_dir, cache_dir):
    helper_script_path = os.path.join(
        os.path.dirname(os.path.realpath(__file__)),
        "cache-convocations-for-day.sh")
    subprocess.run(
        [
            helper_script_path,
            build_dir,
            raw_logs_dir,
            date_to_string(date - datetime.timedelta(days=1)),
            date_to_string(date),
            date_to_string(date + datetime.timedelta(days=1)),
            os.path.join(cache_dir, date_to_string(date) + ".json"),
        ], check=True, timeout=60)


cache_convocations_for_day(
    datetime.date(2020, 1, 1),
    "/Users/johnsullivan/personal/shmeppy-metrics/build",
    "/Users/johnsullivan/personal/shmeppy-admin/shmeppy-metrics/raw-logs",
    "/tmp/testcache")
