use std::cmp::Ordering;
use std::collections::VecDeque;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

struct Floor {
  arrived: VecDeque<Person>,
  up: VecDeque<Person>,
  down: VecDeque<Person>,
}

impl Floor {
  pub fn new() -> Floor {
    let arrived = VecDeque::new();
    let up = VecDeque::new();
    let down = VecDeque::new();
    Floor { arrived: arrived, up: up, down: down }
  }

  pub fn add_up(&mut self, person: Person) {
    self.up.push_back(person);
  }

  pub fn add_down(&mut self, person: Person) {
    self.down.push_back(person);
  }

  pub fn add_arrived(&mut self, person: Person) {
    self.arrived.push_front(person);
  }

  pub fn has_waiting(&self) -> bool {
    !(self.up.is_empty() && self.down.is_empty())
  }

  pub fn get_waiting_for_direction(&mut self, direction: & Direction) -> & VecDeque<Person> {
    match *direction {
      Direction::Up => & self.up,
      Direction::Down => & self.down,
      Direction::None => & self.arrived,
    }
  }
}

#[derive(Debug)]
enum Direction {
  Up,
  None,
  Down,
}

impl Direction {
  pub fn swap(&mut self) {
    match *self {
      Direction::Up => *self = Direction::Down,
      Direction::Down => *self = Direction::Up,
      _ => *self = Direction::None,
    }
  }
}

#[derive(Clone)]
struct Person {
  destination: usize,
  name: String,
}

impl Person {
  fn direction(&self, current_floor: usize) -> Direction {
    match current_floor.cmp(&self.destination) {
      Ordering::Less => Direction::Up,
      Ordering::Equal => Direction::None,
      Ordering::Greater => Direction::Down,
    }
  }
}


struct Elevator {
  floor: usize,
  people: Vec<Person>,
}

impl Elevator {
  pub fn new(capacity: usize) -> Elevator {
    let people = Vec::with_capacity(capacity);
    Elevator { floor: 0, people: people }
  }

  pub fn set_floor(&self, floor: usize) -> Elevator {
    Elevator { floor: floor, people: self.people.clone() }
  }

  pub fn move_direction(&mut self, direction: & Direction) {
    self.floor = match *direction {
      Direction::Up => self.floor + 1,
      Direction::Down => self.floor - 1,
      _ => 0,
    }
  }

  pub fn move_up(&mut self) {
    self.move_direction(&Direction::Up);
  }

  pub fn move_down(&mut self) {
    self.move_direction(&Direction::Down);
  }

  pub fn has_space(&self) -> bool {
    (self.people.len() < self.people.capacity())
  }
}

struct Building {
  floors: Vec<Floor>,
}

impl Building {
  pub fn new(floor_count: usize) -> Building {
    let mut floors = Vec::with_capacity(floor_count);

    for _ in 0..floor_count {
      floors.push(Floor::new());
    }

    Building { floors: floors }
  }

  pub fn is_empty(&self) -> bool {
    for floor in &self.floors {
      if floor.has_waiting() {
        return false;
      }
    }
    true
  }
}

struct Sim {
  building: Building,
  elevator: Elevator, 
}

impl Sim {
  pub fn new() -> Sim {
    let building = Building::new(5);
    let elevator = Elevator::new(2);

    Sim { building: building, elevator: elevator }
  }

  pub fn run(&mut self) -> usize {
    let mut count: usize = 0;
    let mut direction = Direction::Up;

    // While there are still people waiting to use the elevator
    while !self.building.is_empty() || !self.elevator.people.is_empty() {
      let current_floor = self.elevator.floor;

      // Check which direction to go
      // Currently just goes up and down (cocktail shaker)
      if !self.can_move_direction(&direction) {
        direction.swap();
      }

      // Dismiss Passengers
      let mut to_dismiss_indexes = Vec::with_capacity(self.elevator.people.capacity());

      for (i, person) in self.elevator.people.iter().enumerate() {
        if let Ordering::Equal = person.destination.cmp(&current_floor) {
          to_dismiss_indexes.push(i); 
        }
      }

      for person_index in &to_dismiss_indexes {
        let person = self.elevator.people.remove(*person_index);
        println!("{} got off the elevator at floor {}", person.name, current_floor);
        self.building.floors[current_floor].add_arrived(person);
      }

      // Pickup new passengers
      // TODO: Elevator will continue grab people when it originally has space and runs out
      if self.elevator.has_space() {
        match direction {
          Direction::Up => {
            while self.elevator.has_space() && !self.building.floors[current_floor].up.is_empty() {
              // If there is a person there (Could be a while, then not do the empty check above, I think)
              if let Some(person) = self.building.floors[current_floor].up.pop_front() {
                println!("{} gets on the elevator at floor {}", person.name, current_floor);
                self.elevator.people.push(person);
              }
            }
          },
          Direction::Down => {
            while self.elevator.has_space() && !self.building.floors[current_floor].down.is_empty() {

              if let Some(person) = self.building.floors[current_floor].down.pop_front() {
                println!("{} gets on the elevator at floor {}", person.name, current_floor);
                self.elevator.people.push(person);
              }
            }
          },
          _ => {},
        }
      }

      // Move and increment the count
      self.elevator.move_direction(&direction);
      println!("Elevator moving {:?}", direction);
      count += 1;
    }

    count
  }

  pub fn can_move_direction(&self, direction: & Direction) -> bool {
    match *direction {
      Direction::Up => self.can_move_up(),
      Direction::Down => self.can_move_down(),
      _ => true,
    }
  }

  pub fn can_move_up(&self) -> bool {
    self.elevator.floor < (self.building.floors.capacity() - 1)
  }

  pub fn can_move_down(&self) -> bool {
    self.elevator.floor > 0
  }

}

impl Sim {
  pub fn add_person(&mut self, floor: & usize, person: Person) {
    // TODO: Build a request

    match person.direction(*floor) {
      Direction::Up => { self.building.floors[*floor].add_up(person); },
      Direction::Down => { self.building.floors[*floor].add_down(person); },
      Direction::None => {},
    }
  }
}

fn load_people_from_file(file_path: &str, sim: &mut Sim) -> bool {
  
  // Populate sim with people
  let file_path = Path::new(file_path);
  let mut file = match File::open(&file_path) {
    Ok(file) => file,
    Err(_) => { return false; },
  };

  let mut text = String::new();
  file.read_to_string(&mut text);

  let mut lines = text.lines();
  let count: usize = lines.next().unwrap().parse().unwrap();

  for _ in 0..count {
    let mut properties = lines.next().unwrap().split_whitespace();

    let destination: usize = properties.next().unwrap().parse().unwrap();
    let name = String::from(properties.next().unwrap());
    let starting_floor: usize = properties.next().unwrap().parse().unwrap();

    let p = Person { destination: destination, name: name };
    sim.add_person(&starting_floor, p);
  }
  true
}

fn main() {

  let mut sim = Sim::new();
  let loaded = load_people_from_file("people.txt", &mut sim);

  if !loaded {
    // Prompt for people to populate
    panic!("File 'people.txt' not found");
  }
  
  // Run and display the count
  let count = sim.run();
  println!("{}", count);
}
