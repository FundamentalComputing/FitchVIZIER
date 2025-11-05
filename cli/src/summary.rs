use comfy_table::*;
use fitch_proof::*;

type SummaryTable = Vec<(&'static str, Vec<(&'static str, usize)>)>;

struct Summary {
    table : SummaryTable,
    line_count : i32,
    max_depth : usize
}

fn initial_table() -> SummaryTable {
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

impl Summary {
    fn merge <'a>(self: &'a mut Summary, other: &Summary) -> &'a mut Summary {
        merge_summary_tables(&mut self.table, &other.table);
        if other.max_depth > self.max_depth { self.max_depth = other.max_depth };
        self.line_count += other.line_count;
        self
    }
}

fn collect_summary(nodes: &[ProofNode]) -> Summary {
    let mut line_count = 0;
    let mut rules_used = initial_table();
    let mut max_depth = 0;
    
    // collect the table of all the used rules and the max depth
    for line in nodes.iter().filter_map(ProofNode::as_numbered) {
        line_count += 1;
        if line.depth > max_depth { max_depth = line.depth };
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
    Summary { table : rules_used, line_count, max_depth }
}


fn print_summary(s : Summary) {    
    let Summary { table: rules_used, line_count, max_depth } = s;

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
    println!("Max proof depth: {max_depth}");
    println!("Total numbered lines: {line_count}")
}

pub fn summaries_files(files: &Vec<String>) {
    let mut summary = Summary { table: initial_table(), line_count: 0, max_depth: 0 };

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
        let summary1 = collect_summary(&nodes);
        summary.merge(&summary1);
    }
    print_summary(summary);
}
