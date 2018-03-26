use std::collections::BTreeMap;
use std::cmp::{Ordering, min};
use std::cmp::Ordering::*;

use ballot::*;
use candidate::*;
use group::Group;
use std::error::Error;

pub use self::BallotParseErr::*;
pub use self::InvalidBallotErr::*;
pub use self::ChoiceConstraint::*;
pub use self::CountConstraint::*;

#[derive(Debug)]
pub enum BallotParseErr {
    InvalidBallot(InvalidBallotErr),
    InputError(Box<Error>),
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum InvalidBallotErr {
    InvalidCharacter,
    InvalidMinAbove(usize),
    InvalidMaxAbove(usize),
    InvalidMinBelow(usize),
    InvalidMaxBelow(usize),
    InvalidStrict,
    EmptyBallot,
}

/// This type is yielded from iterators used during ballot parsing.
///
/// It allows us to capture GVT multi-votes, and handle the two different types of errors:
///     1. Ballot parsing errors, which are recoverable (skip the ballot).
///     2. IO errors, CSV parsing errors, which are not recoverable (stop the algorithm).
pub type IOBallot = Result<Ballot, BallotParseErr>;

#[derive(Clone, Copy)]
pub enum ChoiceConstraint {
    Strict,
    PreferAbove,
    PreferBelow,
}

#[derive(Clone, Copy)]
pub enum CountConstraint {
    MinAbove(usize),
    MaxAbove(usize),
    MinBelow(usize),
    MaxBelow(usize),
}

pub struct Constraints {
    pub choice: ChoiceConstraint,
    pub counts: Vec<CountConstraint>,
}

impl Constraints {
    // Preferring below the line votes is codified in Section 269(2) of the Electoral Act.
    pub fn official() -> Constraints {
        Constraints {
            choice: PreferBelow,
            counts: vec![MinAbove(1), MinBelow(6)],
        }
    }

    fn check_cmp<F>(
        invalid: Ordering,
        vote_length: usize,
        val: usize,
        err: F,
    ) -> Result<(), BallotParseErr>
    where
        F: Fn(usize) -> InvalidBallotErr,
    {
        if vote_length.cmp(&val) == invalid {
            Err(InvalidBallot(err(vote_length)))
        } else {
            Ok(())
        }
    }

    fn check_min<F>(vote_length: usize, min: usize, err: F) -> Result<(), BallotParseErr>
    where
        F: Fn(usize) -> InvalidBallotErr,
    {
        // good if: vote_length >= min, bad if: vote_length < min
        Constraints::check_cmp(Less, vote_length, min, err)
    }

    fn check_max<F>(vote_length: usize, max: usize, err: F) -> Result<(), BallotParseErr>
    where
        F: Fn(usize) -> InvalidBallotErr,
    {
        // good if: vote_length <= max, i.e. bad if vote_length > max
        Constraints::check_cmp(Greater, vote_length, max, err)
    }

    /// Validate an above the line vote.
    fn check_above<'a>(&self, vote: GroupPrefMap<'a>) -> Result<GroupPrefMap<'a>, BallotParseErr> {
        for &count_constraint in &self.counts {
            match count_constraint {
                MinAbove(min) => Constraints::check_min(vote.len(), min, InvalidMinAbove)?,
                MaxAbove(max) => Constraints::check_max(vote.len(), max, InvalidMaxAbove)?,
                _ => (),
            }
        }
        Ok(vote)
    }

    fn check_below(&self, vote: PrefMap) -> Result<PrefMap, BallotParseErr> {
        for &count_constraint in &self.counts {
            match count_constraint {
                MinBelow(min) => Constraints::check_min(vote.len(), min, InvalidMinBelow)?,
                MaxBelow(max) => Constraints::check_max(vote.len(), max, InvalidMaxBelow)?,
                _ => (),
            }
        }
        Ok(vote)
    }
}

fn remove_repeats_and_gaps<T>(
    (mut map, cutoff): BallotRes<T>,
) -> Result<BTreeMap<u32, T>, BallotParseErr> {
    // Search for a gap in the order of preferences.
    let missing_pref = map.keys()
        .zip(1..)
        .find(|&(&pref, idx)| pref != idx)
        .map(|(_, idx)| idx);

    // Cut-off at the minimum of the provided cutoff (for doubled prefs) and any missing pref.
    let new_cutoff = match (cutoff, missing_pref) {
        (Some(prev), Some(new)) => Some(min(prev, new)),
        (x @ Some(_), _) | (_, x) => x,
    };

    if let Some(cut) = new_cutoff {
        map.split_off(&cut);
    }

    if !map.is_empty() {
        Ok(map)
    } else {
        Err(InvalidBallot(EmptyBallot))
    }
}

pub fn parse_ballot_str(
    pref_string: &str,
    groups: &[Group],
    candidates: &[CandidateId],
    constraints: &Constraints,
    experiment_num: usize,
) -> IOBallot {
    // Iterator over integer preferences.
    let mut pref_iter = pref_string.split(',');

    let above_the_line = create_group_pref_map(pref_iter.by_ref().take(groups.len()), groups)
        .and_then(remove_repeats_and_gaps)
        .and_then(|v| constraints.check_above(v))
        .map(|ok| flatten_group_pref_map(ok, experiment_num));

    // for abl in above_the_line.iter() {
    //     println!("{:?}", abl);
    // }

    let below_the_line = create_pref_map(pref_iter, candidates)
        .and_then(remove_repeats_and_gaps)
        .and_then(|v| constraints.check_below(v))
        .map(flatten_pref_map);

    match (constraints.choice, above_the_line, below_the_line) {
        (_, Ok(prefs), Err(_)) |
        (_, Err(_), Ok(prefs)) |
        (PreferAbove, Ok(prefs), Ok(_)) |
        (PreferBelow, Ok(_), Ok(prefs)) => Ok(Ballot::single(prefs)),
        (Strict, Ok(_), Ok(_)) => Err(InvalidBallot(InvalidStrict)),
        (_, Err(e1), Err(_)) => Err(e1),
    }
}

/// Mapping from preferences to candidate IDs (below the line voting).
pub type PrefMap = BTreeMap<u32, CandidateId>;

/// Mapping from preferences to groups of candidates (above the line voting).
pub type GroupPrefMap<'a> = BTreeMap<u32, &'a [CandidateId]>;

/// Ballot parse result including a map, and an optional preference cut off.
type BallotRes<T> = (BTreeMap<u32, T>, Option<u32>);

pub fn flatten_pref_map(pref_map: PrefMap) -> Vec<CandidateId> {
    pref_map.into_iter().map(|(_, x)| x).collect()
}

pub fn flatten_group_pref_map(group_pref_map: GroupPrefMap, experiment_num: usize) -> Vec<CandidateId> {
    let size = group_pref_map.values().map(|x| x.len()).sum();
    let mut flat = Vec::with_capacity(size);

    let mut small_parties_first = false;
    if experiment_num == 1 {
        small_parties_first = true;
    }

    let labor = [1058, 1059, 1060, 1061, 1062, 1063, 1064, 1065, 1066, 1067, 1068, 1069, 1177, 1178, 1192, 1193, 1194, 1195, 1196, 1197, 1310, 1311, 1312, 1313, 1314, 1315, 1374, 1375, 1376, 1377, 1378, 1379, 1436, 1437, 1438, 1439, 1440, 1441, 1442, 1443, 1552, 1553, 1554, 1555, 1556, 1557, 1558, 998, 999];
    let libs = [1004, 1005, 1028, 1029, 1031, 1033, 1034, 1036, 1037, 1039, 1202, 1203, 1204, 1205, 1206, 1207, 1208, 1209, 1330, 1331, 1332, 1333, 1334, 1335, 1387, 1388, 1389, 1390, 1391, 1392, 1501, 1503, 1504, 1505, 1506, 1604, 1605, 1606, 1607, 1608, 1609, 1610];

    //
    // TODO: here is where the data looks like this:
    // gpm: {1: [1004, 1005], 2: [1010, 1011], 3: [994, 995], 4: [1006, 1007], 5: [1002, 1003], 6: [998, 999]}
    // Where the [1004, 1005] are potential senators from the SAME party
    // I'm guessing the order (1, 2, 3, ...) is which one people put first

    //for (idx, group) in group_pref_map {
    //    println!("{}, {:?}", idx, group);
    //}

    // println!("\nNew vote, size: {}", size);


    let mut found = false;
    let mut found_lib : u32 = 0;
    let mut found_lab : u32 = 0;

    for (idx, group) in &group_pref_map {
        if small_parties_first { 
            for grp in group.iter() {
                if labor.contains(&grp) {
                    found_lab = *idx;
                    found = true;
                    break
                }
                else if libs.contains(&grp) {
                    found_lib = *idx;
                    found = true;
                    break
                }
            }

            if found == false {
                // Let the other parties go first
                flat.extend_from_slice(group);
            }
            found = false;
        } else {
            flat.extend_from_slice(group);
        }
    }

    if small_parties_first { 
        if found_lib > 0 && found_lab > 0 {
            let lib_cans = group_pref_map.get(&found_lib).unwrap();
            let lab_cans = group_pref_map.get(&found_lab).unwrap();

            // found both major parties
            if found_lib < found_lab { 
                // libs are preferred over lab 
                flat.extend_from_slice(lib_cans);
                flat.extend_from_slice(lab_cans);
            } else {
                // lab is preffered over libs
                flat.extend_from_slice(lab_cans);
                flat.extend_from_slice(lib_cans);
            }
        } else if found_lib > 0 {
            // found only the libs
            let lib_cans = group_pref_map.get(&found_lib).unwrap();
            flat.extend_from_slice(lib_cans);
        } else if found_lab > 0 {
            // found only lab
            let lab_cans = group_pref_map.get(&found_lab).unwrap();
            flat.extend_from_slice(lab_cans);
        }
    }

    flat
}

fn create_group_pref_map<'a, 'g, P>(
    prefs: P,
    groups: &'g [Group],
) -> Result<BallotRes<&'g [CandidateId]>, BallotParseErr>
where
    P: Iterator<Item=&'a str>,
{
    let group_candidates = |idx| {
        let group: &'g Group = &groups[idx];
        group.candidate_ids.as_slice()
    };
    create_map(prefs, group_candidates)
}

fn create_pref_map<'a, P>(
    prefs: P,
    candidates: &[CandidateId],
) -> Result<BallotRes<CandidateId>, BallotParseErr>
where
    P: Iterator<Item=&'a str>,
{
    create_map(prefs, |idx| candidates[idx])
}

fn create_map<'a, F, T, P>(prefs: P, func: F) -> Result<BallotRes<T>, BallotParseErr>
where
    F: Fn(usize) -> T,
    P: Iterator<Item=&'a str>,
{
    let mut map = BTreeMap::new();
    let mut pref_cutoff = None;

    for (index, raw_pref) in prefs.enumerate() {

        let pref = match raw_pref {
            "" => continue,
            "*" | "/" => 1,
            _ => {
                raw_pref
                    .parse::<u32>()
                    .map_err(|_| InvalidBallot(InvalidCharacter))?
            }
        };

        let value = func(index);
        let prev_value = map.insert(pref, value);
        //println!("pref: {:?}, index: {:?}", pref, index);

        // If a preference is repeated, we ignore that preference and any
        // higher numbered preferences.
        // Sections 268A(2)(b)(i) and 269(1A)(b)(i).
        if prev_value.is_some() {
            pref_cutoff = Some(match pref_cutoff {
                Some(cutoff) => min(cutoff, pref),
                None => pref,
            });
        }
    }

    Ok((map, pref_cutoff))
}

#[cfg(test)]
mod test {
    use super::remove_repeats_and_gaps;
    use std::collections::BTreeMap;
    use std::iter::FromIterator;

    #[test]
    fn remove_gaps() {
        let mut pref_map = BTreeMap::from_iter((1..10).zip(1..10));
        pref_map.insert(11, 11);

        assert_eq!(
            remove_repeats_and_gaps((pref_map.clone(), None))
                .unwrap()
                .len(),
            9
        );
        assert_eq!(
            remove_repeats_and_gaps((pref_map.clone(), Some(10)))
                .unwrap()
                .len(),
            9
        );
        assert_eq!(
            remove_repeats_and_gaps((pref_map.clone(), Some(5)))
                .unwrap()
                .len(),
            4
        );
    }
}
