#![deny(missing_docs)]

//! MaxSAT Solver

use rsat::cdcl::{Solver, SolverOptions};
use solhop_types::dimacs::Dimacs;
use solhop_types::{Lit, Solution, Var};

/// Solves the maxsat (unweighted) formula and returns the solution along with the optimum
pub fn solve(dimacs: Dimacs) -> (Solution, usize) {
    match dimacs {
        Dimacs::Cnf { n_vars, clauses } => {
            let n_clauses = clauses.len();
            let mut sat_solver = Solver::new(SolverOptions::default());
            let _vars = (0..n_vars)
                .map(|_| sat_solver.new_var())
                .collect::<Vec<_>>();

            let mut ref_vars = vec![];
            let mut cost = clauses.len();

            for clause in clauses {
                let mut clause = clause;
                let ref_var = sat_solver.new_var();
                ref_vars.push(ref_var);
                clause.push(ref_var.pos_lit());
                sat_solver.add_clause(clause);
            }

            if n_clauses == 0 {
                return (sat_solver.solve(vec![]), 0);
            }

            let totalizer_lits = gen_totalizer(&ref_vars, &mut sat_solver);

            let mut last_best = None;

            loop {
                let sol = sat_solver.solve(vec![]);
                match sol {
                    Solution::Unsat => break,
                    Solution::Best(_) => break,
                    Solution::Sat(model) => last_best = Some((model, cost)),
                    Solution::Unknown => break,
                }
                if cost == 0 {
                    break;
                }
                cost -= 1;
                sat_solver.add_clause(vec![!totalizer_lits[cost]]);
            }

            match last_best {
                Some((model, cost)) => (
                    Solution::Best(model[0..n_vars].iter().copied().collect()),
                    n_clauses - cost,
                ),
                None => (Solution::Unknown, 0),
            }
        }
        _ => panic!("not implemented for wcnf yet!"),
    }
}

fn gen_totalizer(vars: &[Var], solver: &mut Solver) -> Vec<Lit> {
    debug_assert!(vars.len() >= 1);
    let mut output: Vec<Vec<Lit>> = vars.into_iter().map(|&v| vec![v.pos_lit()]).collect();
    loop {
        output = totalizer_single_level(output, solver);
        if output.len() == 1 {
            break output[0].clone();
        }
    }
}

fn totalizer_single_level(input: Vec<Vec<Lit>>, solver: &mut Solver) -> Vec<Vec<Lit>> {
    let mut output = vec![];
    let mut input_iter = input.into_iter();
    loop {
        if let Some(first) = input_iter.next() {
            if let Some(second) = input_iter.next() {
                let a = first.len();
                let b = second.len();
                let parent_lits: Vec<_> = (0..a + b).map(|_| solver.new_var().pos_lit()).collect();
                for i in 0..a {
                    solver.add_clause(vec![!first[i], parent_lits[i]]);
                }
                for j in 0..b {
                    solver.add_clause(vec![!second[j], parent_lits[j]]);
                }
                for i in 0..a {
                    for j in 0..b {
                        solver.add_clause(vec![!first[i], !second[j], parent_lits[i + j + 1]]);
                    }
                }
                for i in 1..parent_lits.len() {
                    solver.add_clause(vec![!parent_lits[i], parent_lits[i - 1]]);
                }
                output.push(parent_lits);
            } else {
                output.push(first);
                break;
            }
        } else {
            break;
        }
    }
    output
}
