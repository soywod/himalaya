use serde::Deserialize;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::tui::model::TuiAction;

use std::collections::HashMap;

// ==============
// Constants
// ==============

// Here are all special keybindings, which we can handle. Here are all
// special keys listed:
// https://docs.rs/crossterm/0.19.0/crossterm/event/enum.KeyCode.html
const SPECIALS: [&str; 15] = [
    "BS", "CR", "Left", "Right", "Up", "Down", "Home", "End", "PageUp",
    "PageDown", "Tab", "BackTab", "Delete", "Insert", "Esc",
];

// ==========
// Enums
// ==========
#[derive(Debug, Deserialize, Clone)]
pub enum KeyType {
    Action(Event, TuiAction),
    Key(Event, Vec<KeyType>),
}

// Errors
pub enum KeybindingError {
    NodeConflict,
    ConvertError,
}

// ============
// Structs
// ============
#[derive(Debug, Deserialize)]
pub struct TuiConfig {
    pub sidebar: BlockDataConfig,
    pub mail_list: BlockDataConfig,

    /// Key = Action
    /// Value = Keybinding
    pub keybindings: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct BlockDataConfig {
    pub border_type: Option<String>,
    pub borders: Option<String>,
    pub border_color: Option<String>,
}

impl TuiConfig {
    /// This function will go through all keybindings in  TuiConfig.keybindings
    /// and converts them to a HashMap<Event, KeyType> for the TUI.
    ///
    /// In other words:
    /// It will return the final "form" to lookup after a keybinding.
    pub fn parse_keybindings(&self) -> Vec<KeyType> {
        // Here are all default keybindings stored in the following order:
        //
        //  default_actions = [
        //      (
        //          <action name from config file>,
        //          <action name for the tui>,
        //          <default keybinding>
        //      ),
        //      (
        //          ...
        //      ),
        //      ...
        //
        //  So if you want to add more actions for the TUI or change a default
        //  keybinding, do it in this vector.
        //
        let default_actions = vec![
            ("quit", TuiAction::Quit, "q"),
            ("cursor_down", TuiAction::CursorDown, "j"),
            ("cursor_up", TuiAction::CursorUp, "k"),
        ];

        // This variable will store all keybindings which got converted into
        // <Event, Action>.
        let mut keybindings: Vec<KeyType> = Vec::new();

        // Now iterate through all available actions and look, which one got
        // overridden.
        for action_name in default_actions {
            // Which keybinding has to be add to the keybinding-tree?
            let keybinding = match self.keybindings.get(action_name.0) {
                // The user provided his own keybinding => Add his/her
                // keybinding
                Some(keybinding) => keybinding,

                // The user didn't provide a keybinding => Use the default one.
                None => action_name.2.clone(),
            };

            let mut iter = keybinding.chars();

            // This should rather fungate as a pointer to a node of the
            // keybinding-tree.
            let mut node: &mut Vec<KeyType> = &mut keybindings;

            // TODO: Use a function here, which returns the given event, which
            // has to be added to the tree instead of only one character.
            for key in iter.clone() {
                let event =
                    TuiConfig::convert_to_event(KeyModifiers::NONE, key);

                // If we reached the end of the keybinding-sequence like
                //
                //            g
                //             \
                //   gnn   =    n
                //     ^         \
                //     |          n <- node
                //    node
                //
                // Than we can apply the action to it.
                if iter.as_str().len() == 1 {
                    if let Err(next_index) =
                        node.binary_search_by(|node_event| {
                            let node_event = match node_event {
                                KeyType::Action(eve, _) => eve,
                                KeyType::Key(eve, _) => eve,
                            };

                            match node_event.partial_cmp(&event) {
                                Some(output) => output,
                                None => {
                                    panic!("This shouldn't have happened 2...")
                                }
                            }
                        })
                    {
                        node.insert(
                            next_index,
                            KeyType::Action(event, action_name.1.clone()),
                        );
                    } else {
                        panic!("Bruh");
                    }
                }
                // Suppose we have already stored the following keymapping:
                //
                //  gna
                //
                // Now we'd like to add the following keymapping:
                //
                //  gnn
                //
                // So we've to travel to node `n` first, in order to add the
                // second `n` to `gn`.
                // That's the usage of this else-if-clause: It will let the
                // `node` variable point to the first `n` so it'll look like
                // this:
                //
                // 1.
                //      g  <- node
                //       \
                //        n
                //         \
                //          a
                //
                // 2. (after this else-if-clause)
                //
                //      g
                //       \
                //        n <- node
                //         \
                //          a
                //
                // HINT: This text below might be a little bit wrong, but I
                // it's reason is understandable.
                //
                // We are cloning "node" here, in order to "promise" the
                // compiler, that this else-if-clause doesn't change and
                // nothing bad can happen in the background. So we're using
                // it's clone to get the Hashtable.
                else if let Ok(next_index) =
                    node.binary_search_by(|node_event| {
                        let node_event = match node_event {
                            KeyType::Action(eve, _) => eve,
                            KeyType::Key(eve, _) => eve,
                        };

                        match  node_event.partial_cmp(&event) {
                            Some(output) => output,
                            None => panic!("Bruh"),
                        }
                    })
                {
                    node = if let KeyType::Key(_, next_node) = &mut node[next_index] {
                        next_node
                    }
                    else {
                        panic!("Bruh");
                    }
                }
                // Suppose the user wants to have the following keymapping:
                //
                //  gnn
                //
                // But our keybinding tree looks only like that currently:
                //
                //      g
                //
                // We'd have to create the tree to g->n->n.
                // This else clause is creating the missing nodes to our
                // needed path.
                // So it'll do the following (assuming our tree is like
                // above):
                //
                //      g
                //       \
                //        n <- node
                //
                else {
                    if let Err(next_index) =
                        node.binary_search_by(|node_event| {
                            let node_event = match node_event {
                                KeyType::Action(eve, _) => eve,
                                KeyType::Key(eve, _) => eve,
                            };
                            match node_event.partial_cmp(&event) {
                                Some(output) => output,
                                None => {
                                    panic!("This shouldn't have happened 2...");
                                }
                            }
                        })
                    {
                        // The node is empty => Just push one
                        if next_index == 0 {
                            node.push(KeyType::Key(event, Vec::new()));
                            node = match &mut node[0] {
                                KeyType::Action(_, _) => panic!("Panik!"),
                                KeyType::Key(_, next_node) => next_node,
                            };
                        } else {
                            let tmp_ptr = match &mut node[next_index] {
                                KeyType::Action(_, _) => panic!("Panik!"),
                                KeyType::Key(_, next_node) => next_node,
                            };

                            println!(
                                "len: {}, index: {}",
                                tmp_ptr.len(),
                                next_index
                            );

                            tmp_ptr.insert(
                                next_index,
                                KeyType::Key(event, Vec::new()),
                            );

                            node = match &mut tmp_ptr[next_index] {
                                KeyType::Action(_, _) => panic!("Welp..."),
                                KeyType::Key(_, next_node) => next_node,
                            };
                        }
                    }
                }
                iter.next();
            }
        }
        // Rc::new((*keybindings).clone().into_inner())
        // dbg!("{:?}", keybindings.clone());
        println!("{:?}", keybindings);
        keybindings
    }

    /// This function converts with the given
    /// [code](https://docs.rs/crossterm/0.19.0/crossterm/event/struct.KeyEvent.html#structfield.code)
    /// and
    /// [modifier](https://docs.rs/crossterm/0.19.0/crossterm/event/struct.KeyEvent.html#structfield.modifiers)
    /// its
    /// [KeyEvent](https://docs.rs/crossterm/0.19.0/crossterm/event/struct.KeyEvent.html)
    /// .
    ///
    /// It's just like an alias.
    ///
    /// # Example
    /// ```
    /// # use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    /// // this
    /// let key_event = TuiConfig::convert_to_event(KeyModifiers::NONE, 'c');
    ///
    /// // is the same as this
    /// let key_event2 = Event::Key(KeyEvent {
    ///     modifiers: KeyModifiers::NONE,
    ///     code: KeyCode::Char('c'),
    ///     });
    ///
    /// assert_eq!(key_event, key_event2);
    /// ```
    pub fn convert_to_event(modifiers: KeyModifiers, code: char) -> Event {
        Event::Key(KeyEvent {
            modifiers,
            code: KeyCode::Char(code),
        })
    }

    // // HINT: Not finished yet
    // pub fn get_event_from_keybinding(&self, keybinding: &str) -> Vec<Event> {
    //     let mut events = Vec::new();
    //     let mut iter = keybinding.chars();
    //
    //     let mut special_buffer = String::new();
    //     let mut is_special = false;
    //
    //     // Iterate through the given keybinding and parse it to its
    //     // corresponding event.
    //     //
    //     // Variables:
    //     //  c = character
    //     for c in iter.clone() {
    //         // Did we reach a "special" keybinding?
    //         // Special keybindings are
    //         // keys with a modifier like the Ctrl key and/or the keys in the
    //         // vector of `SPECIALS`.
    //         if c == '<' {
    //             // -----------------------------------
    //             // Collect the special-keybinding
    //             // -----------------------------------
    //             // Collect the special keybinding (if it's really a special
    //             // keybinding)
    //             is_special = true;
    //
    //             special_buffer.extend(iter.take_while(|character| {
    //                 if character.is_none() {
    //                     is_special = false;
    //                 }
    //
    //                 character
    //             }));
    //
    //             // -----------------------
    //             // Check for modifier
    //             // -----------------------
    //             if is_special && special_buffer.len() >= 3 && tmp_c == '>' {
    //                 // Yes it was! Now let's see which kind of
    //
    //                 // Look first, if it has a modifier:
    //             }
    //
    //             // now since we collected the next two chars, look if the
    //             // special buffer looks like this:
    //             //
    //             //  'C-', 'A-' or 'D-'
    //             //
    //             // Because this would mean, that we have a mapping like this:
    //             //
    //             //  <C-...>, <A-...> or <D-...>
    //             //
    //             special_buffer.clear();
    //         }
    //
    //         iter.next();
    //     }
    //
    //     events
    // }

    // pub fn handle_special_keybinding(test_str: &str) -> Result<Event, String> {
    //     let keywords = vec![
    //         "BS", "CR", "Left", "Right", "Up", "Down", "Home", "End", "PageUp",
    //         "PageDown", "Tab", "BackTab", "Delete", "Insert", "Esc",
    //     ];
    //
    //     if test_str.peek() == 'C'
    //         || test_str.peek() == 'A'
    //         || test_str.peek() == 'D'
    //     {}
    // }
}
