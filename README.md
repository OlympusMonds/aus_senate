Australian Senate Voting Experiments
====

This is a fork of the excellent and valuable work done [here][https://github.com/michaelsproul/aus_senate], which is an implementation of the Australian Senate Voting algorithm as described by the AEC.
The AEC won't show us their code, but we can still verify their results using an independent
implementation!

You can read more about this project on [Medium][medium-article].

All code in [Rust][].

# The modifications
I have done my best to modify the code to enable us to run experiments on different voting patterns. So far it mostly supports changing the order of the two major Australian parties to see the effect.

# Running the Code

To download all the CSV files, verify their integrity and run the elections, just do this:

```
$ ./run.py
```

You'll need Python and a Rust compiler.

It will run through 6 experiments detailed in the `run.py` file.

# License

Copyright Michael Sproul 2016. Licensed under the terms of the [GNU General Public License version 3.0 or later][gpl].

[Rust]: https://www.rust-lang.org
[gpl]: https://www.gnu.org/licenses/gpl-3.0.en.html
[medium-article]: https://medium.com/@michaelsproul/how-to-calculate-a-nation-states-election-result-in-your-bedroom-30f0c5d905af
