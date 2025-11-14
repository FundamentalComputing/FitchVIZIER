use comfy_table::*;
use fitch_proof::*;

type SummaryTable = Vec<(&'static str, Vec<(&'static str, usize)>)>;

fn empty_summary() -> SummaryTable {
    vec![
        ("∧", vec![("Intro", 0), ("Elim", 0)]),
        ("∨", vec![("Intro", 0), ("Elim", 0)]),
        ("¬", vec![("Intro", 0), ("Elim", 0)]),
        ("⊥", vec![("Intro", 0), ("Elim", 0)]),
        ("→", vec![("Intro", 0), ("Elim", 0)]),
        ("↔", vec![("Intro", 0), ("Elim", 0)]),
        ("=", vec![("Intro", 0), ("Elim", 0)]),
        ("∀", vec![("Intro", 0), ("Elim", 0)]),
        ("∃", vec![("Intro", 0), ("Elim", 0)]),
        ("R", vec![("Reit", 0)]),
    ]
}

fn collect_summary(nodes: &[ProofNode]) -> (i32, SummaryTable) {
    let mut line_count = 0;
    let mut rules_used = empty_summary();

    for line in nodes.iter().filter_map(ProofNode::as_numbered) {
        line_count += 1;
        if let Some(justification) = &line.justification {
            let (connective, direction) = justification.rule_used();
            if let Some((_, stats))
                = rules_used.iter_mut().find(|(c, _)| *c == connective) {
                if let Some((_, count)) = stats.iter_mut().find(|(d, _)| *d == direction) {
                    *count += 1;
                }
            }
        }
    }
    (line_count, rules_used)
}

fn merge_summary_tables<'a>(base: &'a mut SummaryTable, other: &SummaryTable) -> &'a mut SummaryTable {
    for (connective, stats) in other {
        if let Some((_, base_stats)) = base.iter_mut().find(|(c, _)| *c == *connective) {
            for (direction, count) in stats {
                if let Some((_, base_count)) =
                    base_stats.iter_mut().find(|(d, _)| *d == *direction)
                {
                    *base_count += count;
                } else {
                    base_stats.push((*direction, *count));
                }
            }
        } else {
            base.push((connective, stats.clone()));
        }
    }
    base
}

fn print_summary(line_count: i32, rules_used: SummaryTable) {
    const TABLE_PRESET: &str = "││──╞══╡ ──├┤──┌┐└┘";
    let mut table = Table::new();
    table.load_preset(TABLE_PRESET);
    table.set_header(vec![
        Cell::new("Con "),
        Cell::new("Intro").add_attribute(Attribute::Bold),
        Cell::new("Elim").add_attribute(Attribute::Bold),
    ]);

    for (connective, stats) in rules_used {
        let mut row: Vec<Cell> = vec![];
        row.push(Cell::new(connective).add_attribute(Attribute::Bold));
        for (_dir, count) in stats {
            row.push(Cell::new(format!("{count}")).fg(if count > 0 {
                Color::Green
            } else {
                Color::Red
            }));
        }        
        table.add_row(row);
    }

    println!("{table}");
    println!("Total numbered lines: {line_count}")
}

pub fn summaries_files(files: &Vec<String>) {
    let mut count = 0;
    let mut rules_used: SummaryTable = empty_summary();

    for proof_file in files {
        let Ok(proof) = std::fs::read_to_string(proof_file) else {
            println!(
                "Fatal error! It seems like the file {} could not be opened. Aborting.\n",
                proof_file
            );
            std::process::exit(1)
        };
        let nodes = match parse_fitch_proof(&proof) {
            Ok(nodes) => nodes,
            Err(e) => {
                println!("Fatal error: {}", e);
                std::process::exit(1)
            }
        };
        let (count1, rules_used1) = collect_summary(&nodes);
        count += count1;
        merge_summary_tables(&mut rules_used, &rules_used1);
    }
    print_summary(count, rules_used);
}
