use std::io;
use std::fmt;
use std::panic::resume_unwind;
use std::collections::HashMap;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use rand::rngs::ThreadRng;
use std::arch::x86_64::_mm_sha1rnds4_epu32;
use rand::Rng;


macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}
#[derive(Copy, Clone)]
struct Tree {
    cell_index: i32,
    size: i32,
    is_mine: bool,
    is_dormant: bool,
}

type Forest = HashMap<i32, Tree>;

fn get_forest() -> Forest {
    let mut forest: HashMap<i32, Tree> = HashMap::new();

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let number_of_trees = parse_input!(input_line, i32); // the current amount of trees
    for i in 0..number_of_trees as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let cell_index = parse_input!(inputs[0], i32); // location of this tree
        let size = parse_input!(inputs[1], i32); // size of this tree: 0-3
        let is_mine = parse_input!(inputs[2], i32) == 1; // 1 if this is your tree
        let is_dormant = parse_input!(inputs[3], i32) == 1; // 1 if this tree is dormant
        forest.insert(cell_index, Tree { cell_index, size, is_mine, is_dormant });
    }
    forest
}

#[derive(PartialEq)]
#[derive(Copy, Clone)]
enum Action {
    Grow(i32),
    Seed(i32, i32),
    Complete(i32),
    Wait,
    Null,
}

impl From<&String> for Action {
    fn from(s: &String) -> Self {
        let inputs = s.split(" ").collect::<Vec<_>>();
        match inputs[0] {
            "GROW" => Action::Grow(parse_input!(inputs[1], i32)),
            "SEED" => Action::Seed(parse_input!(inputs[1], i32), parse_input!(inputs[2], i32)),
            "COMPLETE" => Action::Complete(parse_input!(inputs[1], i32)),
            "WAIT" => Action::Wait,
            _ => {
                panic!("Wrong action input");
                Action::Wait
            }
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}",
               match self {
                   Action::Grow(i) => format!("GROW {}", i),
                   Action::Seed(target_id, origin_id) => format!("SEED {} {}", origin_id, target_id),
                   Action::Complete(i) => format!("COMPLETE {}", i),
                   Action::Wait => String::from("WAIT"),
                   Action::Null => String::from("NULL")
               }
        )
    }
}

impl fmt::Display for GameContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "day: {}, sun: {}, nutr: {}, score: {}", self.day, self.sun, self.nutrients, self.score)
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: mine: {}, dormant: {}, size: {}", self.cell_index, self.is_mine, self.is_dormant, self.size)
    }
}


type ActionList = Vec<Action>;

fn get_actionlist() -> ActionList {
    let mut action_list = vec![];
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let number_of_possible_actions = parse_input!(input_line, i32); // all legal actions
    for i in 0..number_of_possible_actions as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let possible_action = input_line.trim_matches('\n').to_string(); // try printing something from here to start with
        action_list.push(Action::from(&possible_action));
    }
    action_list
}

#[derive(Copy, Clone)]
struct GameContext {
    day: i32,
    nutrients: i32,
    sun: i32,
    score: i32,
    op_sun: i32,
    op_score: i32,
    op_is_waiting: bool,
}

fn get_game_context() -> GameContext {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let day = parse_input!(input_line, i32); // the game lasts 24 days: 0-23
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let nutrients = parse_input!(input_line, i32); // the base score you gain from the next COMPLETE action
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(" ").collect::<Vec<_>>();
    let sun = parse_input!(inputs[0], i32); // your sun points
    let score = parse_input!(inputs[1], i32); // your current score
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(" ").collect::<Vec<_>>();
    let op_sun = parse_input!(inputs[0], i32); // opponent's sun points
    let op_score = parse_input!(inputs[1], i32); // opponent's score
    let op_is_waiting = parse_input!(inputs[2], i32) == 1; // whether your opponent is asleep until the next day

    GameContext { day, nutrients, sun, score, op_sun, op_score, op_is_waiting }
}

struct Cell {
    index: i32,
    richness: i32,
    neighbors_ids: Vec<i32>,
}

type Area = HashMap<i32, Cell>;

fn get_area() -> Area {
    let mut area: HashMap<i32, Cell> = HashMap::new();

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let number_of_cells = parse_input!(input_line, i32); // 37
    for i in 0..number_of_cells as usize {
        let mut input_line = String::new();
        let mut neighbors_ids = vec![];
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let index = parse_input!(inputs[0], i32); // 0 is the center cell, the next cells spiral outwards
        let richness = parse_input!(inputs[1], i32); // 0 if the cell is unusable, 1-3 for usable cells
        let neigh_0 = parse_input!(inputs[2], i32); // the index of the neighbouring cell for each direction
        let neigh_1 = parse_input!(inputs[3], i32);
        let neigh_2 = parse_input!(inputs[4], i32);
        let neigh_3 = parse_input!(inputs[5], i32);
        let neigh_4 = parse_input!(inputs[6], i32);
        let neigh_5 = parse_input!(inputs[7], i32);
        for i in 2..8 {
            neighbors_ids.push(parse_input!(inputs[i], i32));
        }
        area.insert(index, Cell { index, richness, neighbors_ids });
    }
    area
}

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/
fn main() {
    let area = get_area();

    let mut previous_action = Action::Null;

    let mut last_day = -1;
    let mut is_new_day = false;

    // let mut calculated_sun = 0;

    // game loop
    loop {
        let context = get_game_context(); // Get input context
        if context.day != last_day {
            is_new_day = true;
            last_day = context.day;
        }


        let mut forest = get_forest(); // Get input forest
        let action_list = get_actionlist(); // List of possible actions

        // eprintln!("--- Action list {} ----", &action_list.len());
        // for s in &action_list {
        //     eprintln!(" * Given Action : {}", s);
        // }
        // eprintln!("----------------");


        let start = Instant::now();
        let calculated_move = playout_moves(&context, &mut forest, &area, &action_list);

        println!("{} score: {} choices: {} ({}) Rolls: {} Time: {}", calculated_move.0, calculated_move.1, calculated_move.2, &action_list.len(), calculated_move.3, start.elapsed().as_millis());
        //println!("{} score: {} choices: {} Rolls: {}", calculated_move.0, calculated_move.1, calculated_move.2, calculated_move.3);
    }

    fn playout_moves(context: &GameContext, mut forest: &mut Forest, board: &Area, expected_actions: &ActionList) -> (String, f64, i32, i32) {
        let mut my_trees_counts = [0, 0, 0, 0];
        let mut opp_trees_counts = [0, 0, 0, 0];
        for tree in forest.values() {
            if tree.is_mine {
                my_trees_counts[tree.size as usize] += 1;
            } else {
                opp_trees_counts[tree.size as usize] += 1;
            }
        }


        let mut rollouts: Vec<(Action, i32)> = vec![];

        let mut number_of_rolls = 0;

        let mut rng = rand::thread_rng();
        let start = Instant::now();

        // for mut i in 1..20 {  //TODO: Add time limit
        let mut i = 0;
        while start.elapsed().as_millis() < 95 {
            i += 1;
            let mut new_game: GameContext = context.clone();
            //eprintln!(" -- New rollout {}, elapsed time: {} --", i, start.elapsed().as_millis());
            // //eprintln!(" -- New  rollout --");
            //eprintln!(" * new_game: {}", new_game);

            // let mut new_previous_action = Action::Null; //last_action.clone();

            let mut new_forest = forest.clone();
            let mut new_last_day = context.day;

            let mut player: usize = 0;

            let mut current_sun_point_balance: [i32; 2] = [context.sun, context.op_sun];
            let mut current_game_points = [context.score, context.op_score];
            let mut current_trees_counts = [my_trees_counts.clone(), opp_trees_counts.clone()];
            let mut waiting_players = [false, false];
            // let mut my_sun_point_balance = context.sun;
            // let mut my_new_game_points = context.score;
            // let mut my_new_trees_counts = my_trees_counts.clone();


            let mut result = choose_next_move(new_game, &mut new_forest, &board, current_sun_point_balance, current_game_points, new_last_day, current_trees_counts, player, waiting_players, number_of_rolls, &mut rng, expected_actions);
            let new_first_action = &result.3.clone();
            // eprintln!(" * EXPECTED new_first_action {}", new_first_action);

            while new_game.day < 24.min(context.day + 10) {
                current_sun_point_balance = result.0;
                current_game_points = result.1;
                new_last_day = result.2;
                new_game = result.4;
                current_trees_counts = result.5;
                waiting_players = result.6;

                player = (player + 1) % 2;

                //eprintln!(" - tried: {} (first action: {})", new_previous_action, new_first_action);
                //eprintln!(" - innerloop day {}  last_day: {}", new_game.day, new_last_day);
                //eprintln!(" - sun_balance: {} game_points: {}", sun_point_balance, new_game_points);

                if new_game.day > 23 && waiting_players[0] && waiting_players[1] {
                    // game over
                    //eprintln!(" ! Reached end of day in innerloop");
                    current_game_points[0] += current_sun_point_balance[0] / 3;
                    //current_game_points[1] += current_sun_point_balance[1] / 3;
                    break;
                }

                result = choose_next_move(new_game, &mut new_forest, &board, current_sun_point_balance, current_game_points, new_last_day, current_trees_counts, player, waiting_players, -1, &mut rng, expected_actions);
                ////eprintln!("innerloop end day {}  last_day: {}", new_game.day, new_last_day);
            }


            rollouts.push((*new_first_action, current_game_points[0]));

            number_of_rolls += 1;

            //eprintln!(" = PUSHED rollout {}, action: {} Score: {}", number_of_rolls, new_first_action, new_game_points);
            //eprintln!("--------------------------------------");
            //eprintln!();
        }


        //eprintln!("Rollouts : {}", rollouts.len());

        let mut number_of_choices = 0;
        // Find the action with max avg score
        let mut results: HashMap<String, (i32, f64)> = HashMap::new();

        for (action, score) in rollouts {
            let action_str = format!("{}", action);

            if results.contains_key(&action_str) {
                let (cur_counts, cur_score) = results.get(&action_str).unwrap();
                results.insert(action_str, (cur_counts + 1, (score as f64 + cur_score * (*cur_counts as f64)) / (cur_counts + 1) as f64));
            } else {
                results.insert(action_str, (1, score as f64));
                number_of_choices += 1;
            }
        }


        // for (a, (c,s)) in &results{
        //     eprintln!(" - a: {} c: {} s: {}", a, c, s);
        // }

        let best_action = results
            .iter()
            .max_by(|(ak, (a_count, a_score)), (bk, (b_count, b_score))| a_score.partial_cmp(&b_score).unwrap())
            .unwrap();//.map(|(k, _v)| k).unwrap();


        (best_action.0.clone(), best_action.1.1, number_of_choices, number_of_rolls)
    }
    fn choose_next_move(mut game: GameContext, trees: &mut Forest, board: &Area, mut current_sun_balance: [i32; 2], mut current_game_points: [i32; 2], mut last_day: i32, mut current_trees_counts: [[i32; 4]; 2], player: usize, mut waiting_players: [bool; 2], roll_number: i32, rng: &mut ThreadRng, expected_actions: &ActionList) -> ([i32; 2], [i32; 2], i32, Action, GameContext, [[i32; 4]; 2], [bool; 2]) {

        //eprintln!(" -- TREES -----");
        // for t in trees.values(){
        //     //eprintln!(" - {}", t);
        // }
        //eprintln!("-------------");
        //eprintln!();


        // Check for seed conflict
        // let (was_seed, target_id, origin_id) = match previous_action {
        //     Action::Seed(i, j) => (true, *i, *j),
        //     _ => (false, 0, 0)
        // };


        // if was_seed && !trees.contains_key(&target_id) {
        //     //eprintln!(" ! Seed conflict! {}", target_id);
        //     sun_balance += get_cost_of_action(&previous_action, &trees, my_trees_counts);
        //     game.nutrients -= 1;
        // }



        if game.day != last_day {
            last_day = game.day;
            let sun_point_calc = get_my_sun_points(&board, &game.day, &trees, player);
            current_sun_balance[player] += sun_point_calc;
            //eprintln!(" ¤¤ new day, got new sun {} (balance: {})", sun_point_calc, sun_balance);

            if player == 0 && game.day < 14 {
                current_game_points[player] += sun_point_calc;
            }
        }

        // --- calculate grow actions ---
        //eprintln!(" SUN before action calculations: {}", sun_balance);
        let mut grow_possibilities: Vec<Action> = calculate_grow_actions(current_sun_balance[player], &trees, current_trees_counts[player], player);

        //eprintln!(" -- GROW possibilities --");
        // for g in &grow_possibilities{
        //     //eprintln!(" * {}", g);
        // }
        //eprintln!();

        // ------------------------------

        // --- calculate complete ations ---

        let mut complete_possibilities: Vec<Action> = calculate_complete_actions(current_sun_balance[player], &trees, player);
        //eprintln!(" -- COMPLETE possibilities --");
        // for g in &complete_possibilities{
        //     //eprintln!(" * {}", g);
        // }
        //eprintln!();

        // ---------------------------------

        // --- calculate seed actions ----


        let roll = rng.gen_range(5..15);
        let mut seed_possibilities: Vec<Action> = vec![];
        let total_trees:i32 = current_trees_counts[player].iter().sum();
        if total_trees < 8 &&  game.day < roll {
            seed_possibilities = calculate_seed_actions(current_sun_balance[player], &trees, &board, current_trees_counts[player], player);
        }
        //eprintln!(" -- SEED possibilities --");
        // for g in &seed_possibilities{
        //     //eprintln!(" * {}", g);
        // }
        //eprintln!();

        // -------------------------------
        let mut possible_choices: Vec<Action> = vec![];




        possible_choices.append(&mut grow_possibilities);
        possible_choices.append(&mut complete_possibilities);
        possible_choices.append(&mut seed_possibilities);

        if (current_sun_balance[player] < 4 || possible_choices.len() == 0) {
            possible_choices.push(Action::Wait);
        }

        if roll_number != -1 {

            possible_choices.sort_by(|a, b| format!("{}",a).partial_cmp(&format!("{}",b)).unwrap());
            // if possible_choices.len() != expected_actions.len() {
            //     eprintln!(" -- POSSIBLE choices {} --", &possible_choices.len());
            //     for g in &possible_choices {
            //         eprintln!(" * {}", g);
            //     }
            //     eprintln!();
            //     eprintln!(" -- EXPECTED: {} --", &expected_actions.len());
            //     for g in expected_actions {
            //         eprintln!(" * {}", g);
            //     }
            //
            //
            // }
        }


        let random_action = match roll_number {
            -1 => possible_choices.choose(rng).unwrap(),
            n => &possible_choices[n as usize % possible_choices.len() ]
        } ;


        let action_cost = get_cost_of_action(&random_action, &trees, current_trees_counts[player]);
        current_sun_balance[player] -= action_cost;

        //eprintln!("random_Action: {}, cost: {} sun: {}", random_action,action_cost,sun_balance);


        if player == 0 {
            current_game_points[player] += calculate_game_points_from_action(&random_action, game.nutrients, &board, &trees, game.day);
        }

        match random_action {
            Action::Grow(cell_index) => {
                let mut tree: Tree = *trees.get(&cell_index).unwrap();
                current_trees_counts[player][tree.size as usize] -= 1;
                tree.size += 1;
                current_trees_counts[player][tree.size as usize] += 1;
                tree.is_dormant = true;
                trees.insert(*cell_index, tree);


                //if player == 0 && game.day < 10 { current_game_points[player] += 2; } //Endorse growing
            }
            Action::Seed(target_cell_index, origin_cell_index) => {
                trees.insert(*target_cell_index, Tree { cell_index: *target_cell_index, size: 0, is_mine: player == 0, is_dormant: true });

                let mut tree: Tree = *trees.get(&origin_cell_index).unwrap();
                tree.is_dormant = true;
                trees.insert(*origin_cell_index, tree);
                current_trees_counts[player][0] += 1;

                //if player == 0 && game.day < 5 { current_game_points[player] += 1; }  //somewhat endorse seeding
            }
            Action::Complete(cell_index) => {
                current_trees_counts[player][trees[cell_index].size as usize] -= 1;
                trees.remove(cell_index);
                game.nutrients -= 1;

                //if player == 0 && game.day > 15 { current_game_points[player] += 4; } //Endorse harvesting in the end
            }
            Action::Wait => {
                waiting_players[player] = true;

                if waiting_players[0] && waiting_players[1] {
                    game.day += 1;
                    //eprintln!("waiting, increase game_day to {}", game.day);

                    for (k, t) in trees {
                        t.is_dormant = false;
                    }
                }
            }
            _ => () //eprintln!("ERROR! Invalid action chosen: {}", random_action)
        }

        return (current_sun_balance, current_game_points, last_day, random_action.clone(), game, current_trees_counts, waiting_players);
    }
    fn calculate_game_points_from_action(action: &Action, nutrients: i32, board: &Area, trees: &Forest, day: i32) -> i32 {
        // let start = Instant::now();

        // eprintln!(" - day {} ACTION: {}",day, action);
        match action {
            Action::Complete(cell_index) => {
                let mut shadow_cost = -calc_shadow_points(*cell_index, &trees, &board, day + 1, false)
                    - calc_shadow_points(*cell_index, &trees, &board, day + 2, false);
                if day > 15{ //
                    shadow_cost += (match board.get(&cell_index).unwrap().richness {
                        2 => nutrients + 2,
                        3 => nutrients + 4,
                        _ => nutrients
                    });
                }
                shadow_cost = shadow_cost+day/20*10;
                return shadow_cost;

            }
            Action::Grow(target_id) => return calc_shadow_points(*target_id, &trees, &board, day + 1, true)
                + calc_shadow_points(*target_id, &trees, &board, day + 2, true)
                + if trees[target_id].size == 0 { 10 } else { 0 }
                + day/24* (5 +trees[target_id].size) ,

            Action::Seed(target_id, origin_id) => {
                let n:i32 = board[target_id].neighbors_ids.iter().filter(|id| !trees.contains_key(id)).map(|x| 1).sum();
                return -n*2
                    + (36-target_id)/10

                    + board[target_id].richness}
            ,
            _ => ()
        }


        // eprintln!(" * calculate_sun_points: {}ms", start.elapsed().as_nanos());
        return 0;
    }
    fn calc_shadow_points(target_id: i32, trees: &Forest, board: &Area, day: i32, action_grow: bool) -> i32 {
        let grown_tree = trees[&target_id];
        let tree_size = grown_tree.size;
        let shadow_direction = day % 6;

        let mut current_id = target_id;

        let mut points = 0;

        for i in 0..tree_size + 1 {
            current_id = board[&current_id].neighbors_ids[shadow_direction as usize];
            if current_id == -1 {
                break;
            }
            if !action_grow || (action_grow && i == tree_size) {
                if trees.contains_key(&current_id) {
                    let tree = trees[&current_id];
                    points += if tree.is_mine {
                        // eprintln!("** {} day {} SHADOWING own tree at {} from {} ({}p.)",day, action_grow, current_id, target_id, -tree.size);
                        -tree.size

                    } else {
                        // eprintln!("** {} day {} SHADOWING opponent tree at {} from {} ({}p.)",day, action_grow, current_id, target_id, tree.size);
                        tree.size
                    }
                }
            }
        }


        points
    }
    fn calculate_seed_actions(sun_points: i32, trees: &Forest, board: &Area, my_trees_counts: [i32; 4], player: usize) -> Vec<Action> {
        // let start = Instant::now();

        let planted_seeds = my_trees_counts[0];

        if sun_points < planted_seeds {
            return vec![];
        }


        let mut result: HashSet<(i32, i32)> = HashSet::new();
        let mut visited: HashSet<i32> = HashSet::new();


        for tree in trees.values() {
            if tree.is_mine && player == 1 || !tree.is_mine && player == 0 {
                continue;
            }
            if tree.is_dormant || tree.size == 0 {
                continue;
            }

            let mut explore_cells: HashSet<i32> = HashSet::new();
            explore_cells.insert(tree.cell_index);

            for _ in 0..tree.size {
                let mut plantable_cells: HashSet<i32> = HashSet::new();
                let mut next_explorable_cells: HashSet<i32> = HashSet::new();

                for ex_cell in explore_cells {
                    let neighbours: Vec<i32> = board[&ex_cell].neighbors_ids.iter().filter(|i| **i != -1 && board[i].richness > 0 && !trees.contains_key(i)).map(|i| *i).collect();
                    // let neighbours: Vec<i32> = board[&ex_cell].neighbors_ids.iter().filter(|i| **i != -1 && board[i].richness > 0 && !trees.contains_key(i) && !visited.contains(i)).map(|i| *i).collect();

                    plantable_cells.extend(&neighbours);
                    // visited.extend(&neighbours);

                    // next_explorable_cells.extend(board[&ex_cell].neighbors_ids.iter().filter(|i| **i != -1 && !visited.contains(i)))
                    next_explorable_cells.extend(board[&ex_cell].neighbors_ids.iter().filter(|i| **i != -1 ))
                }

                result.extend(vec!(tree.cell_index; plantable_cells.len()).into_iter().zip(plantable_cells));
                explore_cells = next_explorable_cells
            }
        }

        let result_actions: Vec<Action> = result
            .iter()
            .map(|(origin_id, target_id)| Action::Seed(*target_id, *origin_id))
            .collect();
        // eprintln!(" * calculate_seed_actions: {}ms", start.elapsed().as_nanos());

        return result_actions;
    }
    fn calculate_complete_actions(sun_points: i32, trees: &Forest, player: usize) -> Vec<Action> {
        // let start = Instant::now();

        if sun_points < 4 {
            return vec![];
        }

        let result: Vec<Action> = trees
            .iter()
            .filter(|(id, tree)| (tree.is_mine && player == 0 || !tree.is_mine && player == 1) && !tree.is_dormant && tree.size == 3)
            .map(|(id, tree)| Action::Complete(tree.cell_index))
            .collect();
        // eprintln!(" * calculate_complete_actions: {}ms", start.elapsed().as_nanos());

        return result;
    }
    fn calculate_grow_actions(sun_points: i32, trees: &Forest, my_trees_counts: [i32; 4], player: usize) -> Vec<Action> {
        // let start = Instant::now();

        let mut tree_costs = [0 + my_trees_counts[0], 1 + my_trees_counts[1], 3 + my_trees_counts[2], 7 + my_trees_counts[3]];


        // for tree in trees.values() {
        //     if tree.is_mine {
        //         tree_costs[tree.size as usize] += 1;
        //     }
        // }


        // for tc in &tree_costs{
        //     //eprintln!("treecost: {}", tc);
        // }

        //eprintln!("sun points: {}", sun_points);

        let result: Vec<Action> = trees
            .iter()
            .filter(|(id, tree)| (tree.is_mine && player == 0 || !tree.is_mine && player == 1) && !tree.is_dormant && tree.size < 3 && tree_costs[(tree.size + 1) as usize] <= sun_points)
            .map(|(id, _)| Action::Grow(*id))
            .collect();

        // eprintln!(" * calculate_grpw_actions: {}ms", start.elapsed().as_nanos());

        return result;
    }
    fn get_cost_of_action(previous_action: &Action, trees: &Forest, my_trees_counts: [i32; 4]) -> i32 {
        return match previous_action {
            Action::Wait => 0,
            Action::Complete(_) => 4,
            Action::Seed(_, _) => my_trees_counts[0],
            Action::Grow(target_cell_id) => {
                let tree_size = trees.get(&target_cell_id).unwrap().size + 1;
                let extra_cost = my_trees_counts[tree_size as usize];


                if tree_size == 1 {
                    return 1 + extra_cost;
                } else if tree_size == 2 {
                    return 3 + extra_cost;
                } else if tree_size == 3 {
                    return 7 + extra_cost;
                } else {
                    //eprintln!("ERROR invalid tree_size {}", tree_size);
                    0
                }
            }
            _ => 0
        };
    }

    fn get_my_sun_points(board: &Area, day: &i32, trees: &Forest, player: usize) -> i32
    {
        // let start = Instant::now();

        // get sun points from trees not in shadow
        let sun_direction = (day + 3) % 6;

        let sun_points =
            trees
                .iter()
                .filter(|(id, tree)| (tree.is_mine && player == 0 || !tree.is_mine && player == 1) && tree.size > 0)
                .map(|(id, tree)| get_sun_points_from_tree(&tree, &board, &trees, sun_direction)).sum();
        // eprintln!(" * get_sun_points: {}ms", start.elapsed().as_nanos());

        return sun_points;
    }
    fn get_sun_points_from_tree(tree: &Tree, board: &Area, trees: &Forest, sun_direction: i32) -> i32 {
        // let start = Instant::now();

        let mut current_tree_id = tree.cell_index;
        for i in 0..3 {
            let neighbor_cell = board.get(&current_tree_id).unwrap().neighbors_ids[sun_direction as usize];

            if neighbor_cell == -1 {
                return tree.size;
            }
            if !trees.contains_key(&neighbor_cell) {
                current_tree_id = neighbor_cell;
                continue;
            }
            if trees[&neighbor_cell].size > i && trees[&neighbor_cell].size >= tree.size {
                return 0;
            }
            current_tree_id = neighbor_cell;
        }

        // eprintln!(" * get_sun_points_from_tree: {}ms", start.elapsed().as_nanos());

        return tree.size;
    }
}