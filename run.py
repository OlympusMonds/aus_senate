#!/usr/bin/env python3

import os
import sys
import json
import subprocess as sp
import fetch_data
from datetime import datetime
from collections import defaultdict

cargo = ["cargo", "run", "--release", "--bin", "election2016", "--"]
#cargo = ["cargo", "run", "--bin", "election2016", "--"]

experiments = {"exp1_no-change" : 1,
               "exp2_bump-1" : 2,
               "exp3_bump-2" : 3,
               "exp4_bump-3" : 4,
               "exp5_bump-4" : 5,
               "exp6_bump-bottom" : 6,}

experiments = {"exp1_no-change" : 1,
               "exp2_10%_chance_bump-1" : 7,
               "exp3_25%_chance_bump-1" : 8,
               "exp4_33%_chance_bump-1" : 9,
               "exp5_50%_chance_bump-1" : 10,
               "exp6_66%_chance_bump-1" : 11,
               "exp7_75%_chance_bump-1" : 12,
               "exp8_90%_chance_bump-1" : 13,
               "exp9_100%_chance_bump-1" : 2,}

results = defaultdict(list)

def run():
    with open("states.json", "r") as f:
        states = json.load(f)

    # If states are specified on the command-line, just run elections for those states.
    if len(sys.argv) > 1:
        states = {s: n for (s, n) in states.items() if s in sys.argv[1:]}

    #fetch_data.fetch(states)

    data_dir = "data"

    candidate_ordering = os.path.join(data_dir, "candidate_ordering.csv")



    for exp_name, exp_id in experiments.items():

        for (state, num_senators) in sorted(states.items()):
            print("Running election for {} at {}".format(state, timestamp()))

            state_csv = os.path.join(data_dir, "{}.csv".format(state))

            args = [candidate_ordering, state_csv, state, str(num_senators), str(exp_id)]

            #output = sp.check_output(cargo + args, stderr=sp.DEVNULL, universal_newlines=True)
            output = sp.check_output(cargo + args, universal_newlines=True)
            # Get party counts
            party_count = defaultdict(int)
            elected_people = output.split("=== Elected ===\n")[-1]
            per_person = elected_people.split("\n")
            for person in per_person:
                if person:
                    party = person.split('{')[-1].split('}')[0]
                    party_count[party] += 1

            json_out = { "state": state, "results": party_count }
            results[exp_name].append(json_out)

            print("Completed election for {} at {}".format(state, timestamp()))

    with open("out.json", 'w') as ok:
        json.dump(results, ok)

def timestamp():
    return datetime.now().isoformat()

if __name__ == "__main__":
    run()
