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

def run():
    with open("states.json", "r") as f:
        states = json.load(f)

    # If states are specified on the command-line, just run elections for those states.
    if len(sys.argv) > 1:
        states = {s: n for (s, n) in states.items() if s in sys.argv[1:]}

    fetch_data.fetch(states)

    data_dir = "data"

    candidate_ordering = os.path.join(data_dir, "candidate_ordering.csv")

    party_count = defaultdict(int)

    for (state, num_senators) in sorted(states.items()):
        print("Running election for {} at {}".format(state, timestamp()))

        state_csv = os.path.join(data_dir, "{}.csv".format(state))

        args = [candidate_ordering, state_csv, state, str(num_senators), '1']

        #output = sp.check_output(cargo + args, stderr=sp.DEVNULL, universal_newlines=True)
        output = sp.check_output(cargo + args, universal_newlines=True)
        # Get party counts
        elected_people = output.split("=== Elected ===\n")[-1]
        per_person = elected_people.split("\n")
        for person in per_person:
            if person:
                party = person.split('(')[-1].split(')')[0]
                party_count[party] += 1

        json_out = { "state": state, "results": party_count }
        print(json.dumps(json_out)) 

        print("Completed election for {} at {}".format(state, timestamp()))

def timestamp():
    return datetime.now().isoformat()

if __name__ == "__main__":
    run()
