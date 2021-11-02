#![feature(proc_macro_hygiene, decl_macro)]

// Modules
#[allow(dead_code)]
mod requests;
#[allow(dead_code)]
mod responses;
#[cfg(test)]
mod test;

// External crates
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate pathfinding;

// Uses
use rocket::http::Status;
use rocket_contrib::json::Json;
use pathfinding::prelude::{absdiff, astar};
use rand::distributions::{Distribution, Uniform};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Pos(i32, i32);
impl Pos {
    fn distance(&self, other: &Pos) -> u32 {
        (absdiff(self.0, other.0) + absdiff(self.1, other.1)) as u32
    }

    fn in_grid(&self) -> u32 {
        if self.0 < 0 || self.0 > 10 || self.1 < 0 || self.1 > 10 {
            return 1000;
        }
        return 0;
    }

    fn in_snake(&self, snakes:&Vec<requests::Snake>, length: u32) -> u32 {
        for snake in snakes.iter() {
            for point in snake.body.iter() {
                if self.0 == point.x && self.1 == point.y {
                    if snake.head.x == self.0 && snake.head.y == self.1 && snake.length < length {
                        return 0;
                    } 
                    return 1000;
                }
            }
        }
        return 0;
    }

    fn closest_food(&self, food:&Vec<requests::Point>) -> Pos {
       if food.len() == 0 { return random_pos() }
       let mut closest_distance = 1000;
       let mut candidate_index = 0;
       for (i,p) in food.iter().enumerate() {
           let distance = self.distance(&Pos(p.x,p.y));
           if distance <= closest_distance {
                candidate_index = i;
                closest_distance = distance;
           }
       }
       return Pos(food[candidate_index].x,food[candidate_index].y)
    }
    
    fn successors(&self) -> Vec<(Pos, u32)> {
        let &Pos(x, y) = self;
        vec![Pos(x+1,y), Pos(x,y+1), Pos(x-1,y), Pos(x,y-1)]
            .into_iter().map(|p| (p, 1)).collect()
    }
}

fn random_pos() -> Pos {
    let mut rng = rand::thread_rng();
    let die = Uniform::from(0..10);
    return Pos(die.sample(&mut rng), die.sample(&mut rng))
}

fn get_movement(start: &Pos, end: &Pos) -> responses::Move {
    // dbg!(start, end);
    if end.0 < start.0 {
        return responses::Move::new(responses::Movement::Left);
    } if end.0 > start.0  {
        return responses::Move::new(responses::Movement::Right);
    } if end.1 > start.1  {
        return responses::Move::new(responses::Movement::Up);
    }
    return responses::Move::new(responses::Movement::Down);
}

#[get("/")]
fn index() -> Json<responses::Info> {
    Json(responses::Info {
        apiversion: "1".to_string(),
        author: None,
        color: Some("#b7410e".to_string()),
        head: None,
        tail: None,
        version: Some("0".to_string()),
    })
}

#[post("/start")]
fn start() -> Status {
    Status::Ok
}

#[post("/move", data = "<req>")]
fn movement(req: Json<requests::Turn>) -> Json<responses::Move> {
    let current_position: Pos = Pos(req.you.head.x,req.you.head.y);
    let food: &Vec<requests::Point> = &req.board.food;
    let goal: Pos = current_position.closest_food(&food);
    let path = astar(&current_position, 
                     |p| p.successors(),
                     |p| p.distance(&goal) + p.in_grid() + p.in_snake(&req.board.snakes,req.you.length),
                     |p| *p == goal);
    // dbg!(food);
    let movement = get_movement(&current_position, &path.unwrap().0[1]);

    Json(movement)
}

#[post("/end")]
fn end() -> Status {
    Status::Ok
}

fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![index, start, movement, end])
}

fn main() {
    rocket().launch();
}
