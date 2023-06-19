use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::fs;
use std::rc::Rc;

struct Node<'a> {
    id: String,
    name: String,
    size: usize,
    is_dir: bool,
    parent: Option<Rc<RefCell<Node<'a>>>>,
    children: Vec<Rc<RefCell<Node<'a>>>>
}

impl<'a> Debug for Node<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("size", &self.size)
            .field("is_dir", &self.is_dir)
            .finish()
    }
}

impl<'a> Node<'a> {
    fn new(id:&str, name:&str, parent: Option<Rc<RefCell<Node<'a>>>>, is_dir: bool, size: usize) -> Node<'a> {
        return Node {
            id: id.to_owned(),
            name: name.to_owned(),
            size,
            is_dir,
            parent,
            children: vec![],
        };
    }
    
    fn get_children(&self) -> Vec<Rc<RefCell<Node<'a>>>> {
        let mut found_children = self.children.iter().map(|child| child.clone()).collect::<Vec<_>>();
        for child in &self.children {
            let mut childs_children = child.borrow().get_children();
            found_children.append(&mut childs_children);
        }
        return found_children;
    }
}


fn is_movement_to_parent(symbols: &Vec<&str>) -> bool {
    return symbols.len() == 3 && symbols[0] == "$" && symbols[1] == "cd" && symbols[2] == ".."
}

fn is_movement_to_child(symbols: &Vec<&str>) -> bool {
    return symbols.len() == 3 && symbols[0] == "$" && symbols[1] == "cd" && symbols[2].is_ascii()
}

fn is_directory(symbols: &Vec<&str>) -> bool {
    return symbols.len() == 2 && symbols[0] == "dir" && symbols[1].is_ascii()
}

fn is_file(symbols: &Vec<&str>) -> bool {
    return symbols.len() == 2 && symbols[0].is_ascii() && symbols[1].is_ascii()
}


fn traverse_lines(lines: &Vec<&str>, root: Rc<RefCell<Node>>) {
    let mut current_node= root;
    for &line in lines {
        let symbols = line.split_ascii_whitespace().collect::<Vec<_>>();

        if is_movement_to_parent(&symbols)  {
            if let Some(parent) = current_node.clone().borrow().parent.clone() {
                current_node = parent
            }
        } else if is_movement_to_child(&symbols) {
            if let Some(child) = current_node.clone().borrow().children.iter().find(|child| child.borrow().name == symbols[2]) {
                current_node = child.clone()
            };
        } else if is_directory(&symbols) {
            let id = current_node.borrow().id.to_owned() + symbols[1] + "/";
            let node = Rc::new(RefCell::new(Node::new(&id, symbols[1], Some(current_node.clone()), true, 0)));
            
            current_node.borrow_mut().children.push(node);
        } else if is_file(&symbols) {
            let size = symbols[0].parse::<usize>().unwrap_or(0);
            let id = current_node.borrow().id.to_owned() + symbols[1];
            let node = Rc::new(RefCell::new(Node::new(&id, symbols[1], Some(current_node.clone()), false, size)));
            
            current_node.borrow_mut().children.push(node);
        }
    }
}

fn main() {
    let data = fs::read_to_string("final-data.txt").unwrap();
    let lines = data.lines().collect::<Vec<_>>();
    
    let root = Rc::new(RefCell::new(Node::new("/", "/", None, true, 0)));
    
    traverse_lines(&lines, root.clone());
    
    let mut nodes = root.borrow().get_children();
    nodes.push(root.clone());
    
    let dirs = nodes.iter().filter(|node| node.borrow().is_dir).collect::<Vec<_>>();
    for dir in &dirs {
        let size = dir.borrow().get_children().iter().filter(|child| !child.borrow().is_dir).map(|child| child.borrow().size).sum::<usize>();
        dir.borrow_mut().size = size;
    }

    const SMALL_DIR_MAX_SIZE:usize = 100000;
    let total_small_dirs_size = dirs.iter().map(|dir| dir.borrow().size).filter(|size| size < &SMALL_DIR_MAX_SIZE).sum::<usize>();
    println!("Total small directories size: {}", total_small_dirs_size);

    const FILESYSTEM_SIZE:usize = 70000000;
    const REQUIRED_SIZE:usize = 30000000;
    
    let used_size = root.borrow().size;
    let unused_size = FILESYSTEM_SIZE - used_size;
    
    let size_to_free_up = REQUIRED_SIZE - unused_size;
    let mut candidate_dir_sizes = dirs.iter().map(|dir| dir.borrow().size).filter(|size| size >= &size_to_free_up).collect::<Vec<_>>();
    candidate_dir_sizes.sort();
    match candidate_dir_sizes.first() {
        None => {}
        Some(best_dir_size) => println!("Size to delete: {}", best_dir_size)
    };
}
