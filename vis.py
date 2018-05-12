import json
from collections import defaultdict
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.ticker import MaxNLocator

def main():
    with open("out.json") as f:
        results = json.load(f)

    # Load in all the results, and collate them into one big list. 
    # Get a unique list of all parties too
    all_parties = set()
    res = []
    for exp in sorted(results.keys()):
        final_senate = defaultdict(int)
        for state in results[exp]:
            state_name = state["state"]
            state_res = state["results"]
            for party, senators in state_res.items():
                all_parties.add(party)
                final_senate[party] += senators

        res.append({"exp": exp, "results": dict(final_senate)})

    
    # Fill out all experiments with all parties. If a party is missing, just give it 0.
    for exp in res:
        existing_parties = set(exp["results"].keys())
        missing_parties = all_parties - existing_parties
        for party in missing_parties:
            exp["results"][party] = 0


    # printing
    for exp in res:
        total_senators = 0
        print(exp["exp"])
        for party, senators in exp["results"].items():
            print("\t", party, senators)
            total_senators += senators
        print("Total senators: ", total_senators)  # Just double checking lolol



    ax = plt.figure().gca()

    # Plotting
    width = 0.12
    cumwidth = 0.
    for exp in res:
        names =  sorted(list(exp["results"].keys()))
        values = [exp["results"][key] for key in names]
        #values = list(exp["results"].values())
        ind = np.arange(len(exp["results"]))
        plt.bar(ind + cumwidth, values, width, label=exp["exp"])
        cumwidth += width

    plt.ylabel('Number of senators')
    plt.title('Effect of bumping the major parties down the preference list')

    plt.xticks(ind + width * 6 / 2, names, rotation=90)
    ax.yaxis.set_major_locator(MaxNLocator(integer=True))
    plt.legend(loc='best')
    plt.show()

if __name__ == "__main__":
    main()
