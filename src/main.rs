// use std::fs::{OpenOptions};
// use std::io::{Write};

use core::time;
use std::{thread, sync::{Arc}};
use indicatif::{ProgressBar, ProgressStyle};

const WIDTH:i32 = 8;
const WHITE:f32 = 1.;
const NONE:f32 = 0.;
const BLACK:f32= -1.;

/* Board Layout
- - - - - - - - - - - - - - -
|0,7|   |   |   |   |   |7,7|
- - - - - - - - - - - - - - - 
|   |   |   |   |   |   |   |
- - - - - - - - - - - - - - - 
|   |   |   |   |   |   |   |
- - - - - - - - - - - - - - -
|   |   |   |   |   |   |   |
- - - - - - - - - - - - - - - 
|   |   |   |   |   |   |   |
- - - - - - - - - - - - - - -
|   |   |   |   |   |   |   |
- - - - - - - - - - - - - - -
|0,1|   |   |   |   |   |   |
- - - - - - - - - - - - - - -
|0,0|1,0|   |   |   |   |7,0|
- - - - - - - - - - - - - - - 
 */


#[derive(Clone, Copy, Debug, PartialEq)]
enum Type {
    None,
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}


#[derive(Clone, Copy, Debug, PartialEq)]
struct Piece {
    p: Type,
    c: f32,
}


#[derive(Clone, Copy, Debug, PartialEq)]
struct Move {
    p0: Piece,
    x0: i32,
    y0: i32,
    p1: Piece,
    x1: i32,
    y1: i32,
    capture: bool,
    promotion: bool,
    enpassant: bool,
    castle: bool,
}

#[derive(Clone, Debug)]
struct Board {
    c: f32,
    b: [Piece; 64],
    moves: Vec<Move>,
    moves_made: Vec<Move>,
    winner: i32,
}

trait Read {
    fn read(&self, x: i32, y: i32) -> Option<Piece>;
}
impl Read for Board {
    fn read(&self, x: i32, y: i32) -> Option<Piece> {
        if x > 7 || x < 0 || y > 7 || y < 0 {
            return None;
        }
        return Some(self.b[(y*WIDTH + x) as usize]);
    }
}

trait BWrite {
    fn write(&mut self, x: i32, y: i32, p: Piece);
}
impl BWrite for Board {
    fn write(&mut self, x: i32, y: i32, p: Piece) {
        self.b[(y*WIDTH + x) as usize] = p;
    }
}

trait Threatened {
    fn threatened(&self, x: i32, y: i32, m: Move) -> bool;
}
// impl Threatened for Board {
//     fn threatened(&self, x: i32, y: i32, m: Move) -> bool{

        
//     }
// }

fn calc_pawn(b:&mut Board, x: i32, y: i32, piece: Piece){
    let color = piece.c;
    let is_last = (y + color as i32)%(WIDTH-1) == 0;
    if let Some(piece_dest) = b.read(x, y + color as i32){
        if piece_dest.p == Type::None {
            // Move Forward One (-1 or 1 depending on color)
            b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x, y1: y + (color as i32), capture: false, promotion: is_last, enpassant: false, castle: false});
            // Move Forward Two (-2 or 2 depending on color)
            if y == (3.5 - 2.5*color) as i32 {
                if let Some(piece_dest) = b.read(x, y + (color as i32)*2) {
                    b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x, y1: y + (color as i32)*2, capture: false, promotion: false, enpassant: false, castle: false});
                }
            }
        }
    }
    // Capture Forward (-1 or 1 depending on color) right
    if let Some(piece_dest) = b.read(x+1, y + color as i32) {
        if piece_dest.p != Type::None && piece_dest.c != color {
            b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x+1, y1: y + (color as i32), capture: true, promotion: is_last, enpassant: false, castle: false});
        }
    }
    // Capture Forward (-1 or 1 depending on color) left
    if let Some(piece_dest) = b.read(x-1, y + color as i32) {
        if piece_dest.p != Type::None && piece_dest.c != color {
            b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x-1, y1: y + (color as i32), capture: true, promotion: is_last, enpassant: false, castle: false});
        }
    }
    // En Passant Left
    if let Some(piece_dest) = b.read(x-1, y + color as i32) {
        if piece_dest.p == Type::None && b.moves_made.len() != 0 {
            if b.moves_made[b.moves_made.len() - 1] == (Move{p0: Piece{p: Type::Pawn, c: color * -1.}, x0: x - 1, y0: y + (color as i32) * 2, p1: Piece{p: Type::None, c: NONE}, x1: x - 1, y1: y, capture: false, promotion: false, enpassant: false, castle: false}) {
                b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x-1, y1: y + (color as i32), capture: false, promotion: false, enpassant: true, castle: false});
            }
        }
    }
    // En Passant Right
    if let Some(piece_dest) = b.read(x+1, y + color as i32) {
        if piece_dest.p == Type::None && b.moves_made.len() != 0 {
            if b.moves_made[b.moves_made.len() - 1] == (Move{p0: Piece{p: Type::Pawn, c: color * -1.}, x0: x + 1, y0: y + (color as i32) * 2, p1: Piece{p: Type::None, c: NONE}, x1: x + 1, y1: y, capture: false, promotion: false, enpassant: false, castle: false}) {
                b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x+1, y1: y + (color as i32), capture: false, promotion: false, enpassant: true, castle: false});
            }
        }
    }
}

fn calc_knight(b:&mut Board, x: i32, y: i32, piece: Piece){
    // Move Right 2 Up 1
    if let Some(piece_dest) = b.read(x + 2, y + 1) {
        if piece_dest.c != piece.c {
            b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x + 2, y1: y + 1, capture: piece_dest.c != NONE, promotion: false, enpassant: false, castle: false});
        }
    }
    // Move Right 2 Down 1
    if let Some(piece_dest) = b.read(x + 2, y - 1) {
        if piece_dest.c != piece.c {
            b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x + 2, y1: y - 1, capture: piece_dest.c != NONE, promotion: false, enpassant: false, castle: false});
        }
    }
    // Move Down 2 Right 1
    if let Some(piece_dest) = b.read(x + 1, y - 2) {
        if piece_dest.c != piece.c {
            b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x + 1, y1: y - 2, capture: piece_dest.c != NONE, promotion: false, enpassant: false, castle: false});
        }
    }
    // Move Down 2 Left 1
    if let Some(piece_dest) = b.read(x - 1, y - 2) {
        if piece_dest.c != piece.c {
            b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x - 1, y1: y - 2, capture: piece_dest.c != NONE, promotion: false, enpassant: false, castle: false});
        }
    }
    // Move Left 2 Down 1
    if let Some(piece_dest) = b.read(x - 2, y - 1) {
        if piece_dest.c != piece.c {
            b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x - 2, y1: y - 1, capture: piece_dest.c != NONE, promotion: false, enpassant: false, castle: false});
        }
    }
    // Move Left 2 Up 1
    if let Some(piece_dest) = b.read(x - 2, y + 1) {
        if piece_dest.c != piece.c {
            b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x - 2, y1: y + 1, capture: piece_dest.c != NONE, promotion: false, enpassant: false, castle: false});
        }
    }
    // Move Up 2 Left 1
    if let Some(piece_dest) = b.read(x - 1, y + 2) {
        if piece_dest.c != piece.c {
            b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x - 1, y1: y + 2, capture: piece_dest.c != NONE, promotion: false, enpassant: false, castle: false});
        }
    }
    // Move Up 2 Left 1
    if let Some(piece_dest) = b.read(x + 1, y + 2) {
        if piece_dest.c != piece.c {
            b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x + 1, y1: y + 2, capture: piece_dest.c != NONE, promotion: false, enpassant: false, castle: false});
        }
    }
}

fn calc_bishop(b:&mut Board, x: i32, y: i32, piece: Piece) {
    // Up Right
    for i in 1..8 {
        if let Some(piece_dest) = b.read(x + i,y + i) {
            if piece_dest.c != piece.c {
                b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x + i, y1: y + i, capture: piece_dest.c != NONE, promotion: false, enpassant: false, castle: false});
            }
            else {
                break;
            }
            // Don't go past a piece
            if piece_dest.c as i32 == piece.c as i32 * -1 {
                break;
            }
        }
        else {
            break;
        }
    }
    // Up Left
    for i in 1..8 {
        if let Some(piece_dest) = b.read(x - i,y + i) {
            if piece_dest.c != piece.c {
                b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x - i, y1: y + i, capture: piece_dest.c != NONE, promotion: false, enpassant: false, castle: false});
            }
            else {
                break;
            }
            // Don't go past a piece
            if piece_dest.c as i32 == piece.c as i32 * -1 {
                break;
            }
        }
        else {
            break;
        }
    }
    // Down Left
    for i in 1..8 {
        if let Some(piece_dest) = b.read(x - i,y - i) {
            if piece_dest.c != piece.c {
                b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x - i, y1: y - i, capture: piece_dest.c != NONE, promotion: false, enpassant: false, castle: false});
            }
            else {
                break;
            }
            // Don't go past a piece
            if piece_dest.c as i32 == piece.c as i32 * -1 {
                break;
            }
        }
        else {
            break;
        }
    }
    // Down Right
    for i in 1..8 {
        if let Some(piece_dest) = b.read(x + i,y - i) {
            if piece_dest.c != piece.c {
                b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x + i, y1: y - i, capture: piece_dest.c != NONE, promotion: false, enpassant: false, castle: false});
            }
            else {
                break;
            }
            // Don't go past a piece
            if piece_dest.c as i32 == piece.c as i32 * -1 {
                break;
            }
        }
        else {
            break;
        }
    }
}

fn calc_rook(b:&mut Board, x: i32, y: i32, piece: Piece) {
    // Up
    for i in 1..8 {
        if let Some(piece_dest) = b.read(x,y + i) {
            if piece_dest.c != piece.c {
                b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x, y1: y + i, capture: piece_dest.c != NONE, promotion: false, enpassant: false, castle: false});
            }
            else {
                break;
            }
            // Don't go past a piece
            if piece_dest.c as i32 == piece.c as i32 * -1 {
                break;
            }
        }
        else {
            break;
        }
    }
    // Left
    for i in 1..8 {
        if let Some(piece_dest) = b.read(x - i,y) {
            if piece_dest.c != piece.c {
                b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x - i, y1: y, capture: piece_dest.c != NONE, promotion: false, enpassant: false, castle: false});
            }
            else {
                break;
            }
            // Don't go past a piece
            if piece_dest.c as i32 == piece.c as i32 * -1 {
                break;
            }
        }
        else {
            break;
        }
    }
    // Down
    for i in 1..8 {
        if let Some(piece_dest) = b.read(x,y - i) {
            if piece_dest.c != piece.c {
                b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x, y1: y - i, capture: piece_dest.c != NONE, promotion: false, enpassant: false, castle: false});
            }
            else {
                break;
            }
            // Don't go past a piece
            if piece_dest.c as i32 == piece.c as i32 * -1 {
                break;
            }
        }
        else {
            break;
        }
    }
    // Right
    for i in 1..8 {
        if let Some(piece_dest) = b.read(x + i,y) {
            if piece_dest.c != piece.c {
                b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x + i, y1: y, capture: piece_dest.c != NONE, promotion: false, enpassant: false, castle: false});
            }
            else {
                break;
            }
            // Don't go past a piece
            if piece_dest.c as i32 == piece.c as i32 * -1 {
                break;
            }
        }
        else {
            break;
        }
    }
}

fn calc_king(b:&mut Board, x: i32, y: i32, piece: Piece) {
    for i in -1..=1 {
        for j in -1..=1 {
            if let Some(piece_dest) = b.read(x + i, y + j) {
                if piece_dest.c != piece.c {
                    b.moves.push(Move{p0: piece, x0: x, y0: y, p1: piece_dest, x1: x + i, y1: y + j, capture: piece_dest.c != NONE, promotion: false, enpassant: false, castle: false});
                } 
            }
        }
    }
}


trait Calculate {
    fn calculate(&mut self);
}
impl Calculate for Board {
    fn calculate(&mut self) {
        self.moves = vec![];
        for i in 0..WIDTH*WIDTH {
            let piece = self.b[i as usize];
            if piece.c == self.c {
                let x = i%WIDTH;
                let y = i/WIDTH;
                match piece.p {
                    Type::None => (),
                    Type::Pawn => {
                        calc_pawn(self, x, y, piece);
                    },
                    Type::Knight => {
                        calc_knight(self, x, y, piece);
                    },
                    Type::Bishop => {
                        calc_bishop(self, x, y, piece);
                    },
                    Type::Rook => {
                        calc_rook(self, x, y, piece);
                    },
                    Type::Queen => {
                        calc_bishop(self, x, y, piece);
                        calc_rook(self, x, y, piece);
                    },
                    Type::King => {
                        calc_king(self, x, y, piece)
                    },
                }
            }
        }
    }
}

trait Evaluate {
    fn evaluate(&self) -> f32;
}
impl Evaluate for Board {
    fn evaluate(&self) -> f32 {
        let mut sum: f32 = 0.0;
        for i in 0..WIDTH*WIDTH {
            sum += match &self.b[i as usize].p{
                Type::None => 0.,
                Type::Pawn => 1.,
                Type::Knight => 3.05,
                Type::Bishop => 3.33,
                Type::Rook => 5.63,
                Type::Queen => 9.5,
                Type::King => 9999.,
            } *&self.b[i as usize].c;
        };
        return sum;
    }
}

fn domove(b: &Board, mo: &Move) -> Board{
    // Open File
    // let file_name = "log.txt";
    // let mut file = OpenOptions::new()
    //     .read(true)
    //     .write(true)
    //     .create(false)
    //     .append(true)
    //     .open(file_name).unwrap();

    // Log Moves into File
    // match write!(file, "{}", format!("{:?}{:?}{:?}\n", m.p0.p, m.x1, m.y1)) {
    //     Ok(_) => (),
    //     Err(_) => println!("Problem writing move {:?}", file),
    // }

    // If Promotion for Pawn
    let m = mo.clone();
    let mut board = b.clone();
    if m.promotion == true{
        board.b[(m.y1*WIDTH + m.x1) as usize] = Piece{p: Type::Queen, c: m.p0.c};
        board.b[(m.y0*WIDTH + m.x0) as usize] = Piece{p: Type::None, c: NONE};
    }
    // Else normal move
    else {
    board.b[(m.y1*WIDTH + m.x1) as usize] = m.p0;
    board.b[(m.y0*WIDTH + m.x0) as usize] = Piece{p: Type::None, c: NONE};
    }
    board.moves_made.push(m);
    if m.p1.p == Type::King {
        board.winner = (m.p1.c * -1.) as i32;
    }
    else {
        board.winner = 0;
    }
    board.c *= -1.;
    return board;
}


fn setup() -> Board {
    let mut b: Board = Board { c: WHITE, b: [Piece{p: Type::None, c: NONE}; 64], moves: vec![], moves_made: vec![], winner: 0};
    // White pieces
    b.write(0, 0, Piece{p: Type::Rook, c: WHITE});
    b.write(1, 0, Piece{p: Type::Knight, c: WHITE});
    b.write(2, 0, Piece{p: Type::Bishop, c: WHITE});
    b.write(3, 0, Piece{p: Type::Queen, c: WHITE});
    b.write(4, 0, Piece{p: Type::King, c: WHITE});
    b.write(5, 0, Piece{p: Type::Bishop, c: WHITE});
    b.write(6, 0, Piece{p: Type::Knight, c: WHITE});
    b.write(7, 0, Piece{p: Type::Rook, c: WHITE});
    b.write(0, 1, Piece{p: Type::Pawn, c: WHITE});
    b.write(1, 1, Piece{p: Type::Pawn, c: WHITE});
    b.write(2, 1, Piece{p: Type::Pawn, c: WHITE});
    b.write(3, 1, Piece{p: Type::Pawn, c: WHITE});
    b.write(4, 1, Piece{p: Type::Pawn, c: WHITE});
    b.write(5, 1, Piece{p: Type::Pawn, c: WHITE});
    b.write(6, 1, Piece{p: Type::Pawn, c: WHITE});
    b.write(7, 1, Piece{p: Type::Pawn, c: WHITE});
    // Black Pieces
    b.write(0, 7, Piece{p: Type::Rook, c: BLACK});
    b.write(1, 7, Piece{p: Type::Knight, c: BLACK});
    b.write(2, 7, Piece{p: Type::Bishop, c: BLACK});
    b.write(3, 7, Piece{p: Type::Queen, c: BLACK});
    b.write(4, 7, Piece{p: Type::King, c: BLACK});
    b.write(5, 7, Piece{p: Type::Bishop, c: BLACK});
    b.write(6, 7, Piece{p: Type::Knight, c: BLACK});
    b.write(7, 7, Piece{p: Type::Rook, c: BLACK});
    b.write(0, 6, Piece{p: Type::Pawn, c: BLACK});
    b.write(1, 6, Piece{p: Type::Pawn, c: BLACK});
    b.write(2, 6, Piece{p: Type::Pawn, c: BLACK});
    b.write(3, 6, Piece{p: Type::Pawn, c: BLACK});
    b.write(4, 6, Piece{p: Type::Pawn, c: BLACK});
    b.write(5, 6, Piece{p: Type::Pawn, c: BLACK});
    b.write(6, 6, Piece{p: Type::Pawn, c: BLACK});
    b.write(7, 6, Piece{p: Type::Pawn, c: BLACK});
    return b
}

fn negamax(mut b: Board, depth: i32, mut alpha: f32, beta: f32) -> f32 {
    if depth == 0 || b.winner != 0{
        return b.c * b.evaluate();
    }
    b.calculate();
    let mut value: f32 = -1. * f32::MAX;
    for m in b.clone().moves {
        value = value.max(-1. * negamax(domove(&b, &m), depth - 1, beta * -1., alpha * -1.));
        alpha = alpha.max(value);
        if alpha >= beta {
            break;
        }  
    }
    return value;
}

fn main() {
    let depth = 6;
    // Create Log File
    // let file_name = "log.txt";
    // let mut file = OpenOptions::new()
    //     .read(true)
    //     .write(true)
    //     .create(true)
    //     .append(false)
    //     .open(file_name).unwrap();

    // Create Board
    let mut b= setup();

    // Calculate moves for the board
    b.calculate();

    println!("Heuristic Score: {}", b.evaluate());

    // Immutable reference across threads
    let arc_b = Arc::new(b.clone());

    // List of threads that we generate
    let mut threads: Vec<_> = vec![];

    // Thread bar cause I'm a lunatic
    let thread_bar = ProgressBar::new((b.clone().moves.len()) as u64);
    thread_bar.set_style(ProgressStyle::with_template("{spinner:.green} [{wide_bar:.green/blue}] [{pos}/{len} Threads Generated]")
    .unwrap()
    .progress_chars("#>-"));

    for i in 0..b.clone().moves.len(){

        // Copy our board data
        let arc_b = Arc::clone(&arc_b);

        // Create a thread that does the negamax function
        let handle = std::thread::spawn(move || {
                let value =  negamax(domove(&arc_b, &arc_b.moves[i]), depth, -999999., 999999.);
                return (value, arc_b.moves[i]);
       
        });
        // Puts this thread into our list of threads
        threads.push(handle);
        thread_bar.inc(1);
    }

    let mut moves: Vec<Move> = vec![];
    let mut values: Vec<f32> = vec![];

    let bar = ProgressBar::new((b.clone().moves.len()) as u64);
    bar.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{wide_bar:.cyan/blue}] [{pos}/{len} Moves]")
    .unwrap()
    .progress_chars("#>-"));
    bar.tick();

    // For each thread wait for them to finish and unwrap their tuple results
    for thread in threads {
        let (v, m) = thread.join().unwrap();
        moves.push(m);
        values.push(v);
        // This is entirely just so the bar looks prettier and is in fact less efficient and stupid
        bar.inc(1);
        thread::sleep(time::Duration::from_millis(20))
    }  
    
    // Find Max
    let mut max_index = 0;
    for i in 0..moves.len() {
        if values[i] > values[max_index] {
            max_index = i;
        }
    }
    println!("Best Move is:");
    println!("{:?} from ({},{}) to ({},{}) with a value of {}", moves[max_index].p0, moves[max_index].x0, moves[max_index].y0, moves[max_index].x1, moves[max_index].y1, values[max_index]);
    
}
