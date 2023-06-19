use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

struct Tree<'a> {
    root: Option<*mut Node<'a>>,
    nodes: Vec<Rc<RefCell<Node<'a>>>>
}

impl<'a> Tree<'a> {
    fn new(root_id:&str, root_name:&str) -> Tree<'a> {
        return Tree {
            root: None,
            nodes: vec![Rc::new(RefCell::new(Node::new(root_id, root_name, None, true, 0)))],
        };
    }
    
    fn create_root(&'a mut self) {
        let node_ptr = self.nodes[0].as_ptr();
        self.root = Option::from(node_ptr);
    }
    
    unsafe fn add_node(&'a mut self, node: Node<'a>) -> *mut Node {
        let node = Rc::new(RefCell::new(node));
        let ptr = node.as_ptr();
        self.nodes.push(node);
        (*(*ptr).parent.unwrap()).children.push(ptr);
        
        let mut par_ptr = (*ptr).parent.unwrap();
        loop {
            (*par_ptr).size += (*ptr).size;
            match (*par_ptr).parent {
                None => break,
                Some(parent) => par_ptr = parent
            }
        }
        return ptr;
    }
    
    unsafe fn get_dirs(&'a mut self) -> Vec<*mut Node> {
        return (*self).nodes.iter().map(|node_ref| node_ref.as_ptr()).filter(|&node_ptr| (*node_ptr).is_dir).collect()
    }
}

#[derive(Debug)]
struct Node<'a> {
    id: String,
    name: String,
    size: usize,
    is_dir: bool,
    parent: Option<*mut Node<'a>>,
    children: Vec<*mut Node<'a>>
}

impl<'a> Node<'a> {
    fn new(id:&str, name:&str, parent: Option<*mut Node<'a>>, is_dir: bool, size: usize) -> Node<'a> {
        return Node {
            id: id.to_owned(),
            name: name.to_owned(),
            size,
            is_dir,
            parent,
            children: vec![],
        };
    }
    
    fn get_child_by_name(&'a self, name: &str) -> Option<&'a *mut Node<'a>> {
        unsafe {
            return self.children.iter().find(|&child| (*(*child)).name == name)
        }
    }
}

fn is_root_creation(symbols: &Vec<&str>) -> bool {
    return symbols.len() == 3 && symbols[0] == "$" && symbols[1] == "cd" && symbols[2] == "/"
}

fn is_listing(symbols: &Vec<&str>) -> bool {
    return symbols.len() == 2 && symbols[0] == "$" && symbols[1] == "ls"
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

unsafe fn traverse_lines(lines: &Vec<&str>, tree_ptr: *mut Tree) {
    let mut current_node:Option<*mut Node> = None;
    for &line in lines {
        let symbols = line.split_ascii_whitespace().collect::<Vec<_>>();
        
        if is_root_creation(&symbols) {
            (*tree_ptr).create_root();
            current_node = (*tree_ptr).root
        } else if is_listing(&symbols)  { 
            //Do nothing
        } else if is_movement_to_parent(&symbols)  {
            match (*(current_node.unwrap())).parent {
                None => {}
                Some(parent) => current_node = Some(parent)
            }
        } else if is_movement_to_child(&symbols)  { 
            match (*(current_node.unwrap())).get_child_by_name(symbols[2]) {
                None => {}
                Some(child) => current_node = Some(*child)
            }
        } else if is_directory(&symbols) {
            let new_id = (&*(*(current_node.unwrap())).id).to_owned() + symbols[1] + "/";
            (*tree_ptr).add_node(Node::new(&new_id, symbols[1], current_node, true, 0));
        } else if is_file(&symbols) {
            let new_id = (&*(*(current_node.unwrap())).id).to_owned() + symbols[1];
            (*tree_ptr).add_node(Node::new(&new_id, symbols[1], current_node, false, symbols[0].parse::<usize>().unwrap_or(0)));
        }
    }
}

fn main() {
    let data = fs::read_to_string("final-data.txt").unwrap();
    let lines = data.lines().collect::<Vec<_>>();
    
    let tree = Tree::new("/", "/");
    let tree_ref:Rc<RefCell<Tree>> = Rc::new(RefCell::new(tree));
    
    unsafe {traverse_lines(&lines, tree_ref.as_ptr())};
    unsafe {
        let dirs = (*tree_ref.as_ptr()).get_dirs();
        
        let small_dirs = dirs.iter().filter(|&&dir| (*dir).size <= 100000).collect::<Vec<_>>();
        let total_small_dirs_size:usize = small_dirs.iter().map(|&&dir| (*dir).size).sum();
        
        let dir_sizes = dirs.iter().map(|&dir| (*dir).size).collect::<Vec<_>>();
        let total_used_size = (*(*tree_ref.as_ptr()).root.unwrap()).size;
        let total_filesystem_size:usize = 70000000;
        let total_unused_size = total_filesystem_size - total_used_size;
        let required_unused_size:usize = 30000000;
        let size_to_free_up = required_unused_size - total_unused_size;
        
        let mut possible_dir_sizes_to_delete = dir_sizes.iter().filter(|&dir_size| dir_size >= &size_to_free_up).collect::<Vec<_>>();
        possible_dir_sizes_to_delete.sort();

        println!("Total small dirs size: {}", total_small_dirs_size);
        println!("Size to delete: {}", possible_dir_sizes_to_delete.first().unwrap())
    }
}
