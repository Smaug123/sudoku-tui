# Starting out

[Initial chat with Sonnet](https://claude.ai/chat/567f3a8a-1ab3-4749-8fd0-56aebd1ef7cb).

# Annoying long boi help-text box

At this point, the help text is displayed in its entirety, but the box fills the entire screen, which is ugly.
I manually decided to refactor to extract the help text, so that the rendering loop knows how big the space should be; I did that with Copilot out of the box.
I pulled out this snippet into a `const` definition:

```rust
vec![
    Line::from(vec![
        Span::styled("Controls", Style::default().fg(Color::White).add_modifier(ratatui::style::Modifier::BOLD)),
    ]),
    Line::from(vec![
        Span::styled("Movement: ", Style::default().fg(Color::White)),
        Span::styled("↑ ↓ ← →", Style::default().fg(Color::Yellow)),
        Span::raw(" arrow keys to navigate the grid"),
    ]),
    Line::from(vec![
        Span::styled("Modes: ", Style::default().fg(Color::White)),
    ]),
    Line::from(vec![
        Span::styled("/", Style::default().fg(Color::Yellow)),
        Span::raw(" - Normal mode (enter numbers directly)"),
    ]),
    Line::from(vec![
        Span::styled(",", Style::default().fg(Color::Yellow)),
        Span::raw(" - Corner mode (small numbers in corners)"),
    ]),
    Line::from(vec![
        Span::styled(".", Style::default().fg(Color::Yellow)),
        Span::raw(" - Center mode (small numbers in center)"),
    ]),
    Line::from(vec![
        Span::styled("Numbers: ", Style::default().fg(Color::White)),
        Span::raw("Use keys 1-9 to enter values"),
    ]),
    Line::from(vec![
        Span::styled("Color coding:", Style::default().fg(Color::White)),
    ]),
    Line::from(vec![
        Span::styled("Green", Style::default().fg(Color::Green)),
        Span::raw(" - Fixed numbers (unchangeable)"),
    ]),
    Line::from(vec![
        Span::styled("White", Style::default().fg(Color::White)),
        Span::raw(" - User-entered numbers"),
    ]),
    Line::from(vec![
        Span::styled("Yellow", Style::default().fg(Color::Yellow)),
        Span::raw(" - Corner numbers (up to 4)"),
    ]),
    Line::from(vec![
        Span::styled("Blue", Style::default().fg(Color::Blue)),
        Span::raw(" - Center numbers (up to 3)"),
    ]),
    Line::from(vec![
        Span::styled("Exit: ", Style::default().fg(Color::White)),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" to quit the application"),
    ]),
];
```

Asked Copilot:

> Defunctionalise this: store the strings and the optional colours, rather than calling `Span::styled` etc. Omit the first entry. For example, the first output should be `vec![("Movement", Some(Color::White)), "(the arrow keys)", Some(Color::Yellow)], (" arrow …", None)`.

I accepted the result verbatim, which failed to compile because you can't allocate `vec!` in const context.
I [asked Sonnet](https://claude.ai/chat/53a72b05-57c6-4186-8381-69b991a82701) how to deal with this, and accepted its recommendation to use `&[...]` rather than `vec![]`.

# `cargo` trampling over itself

While doing this, in switching between VS Code and `cargo run`, I found the two instances of Rust trampling over each other; I [asked Claude](https://claude.ai/chat/53a72b05-57c6-4186-8381-69b991a82701) (second message) how to deal with this, and distrusting its answer I also asked Copilot:

> With rust-analyzer and cargo, I am finding that every `cargo run` causes rust-analyzer to recompile the dependencies of my project, and vice versa. How does one usually avoid this?

I ended up with Copilot's answer (configure the `.vscode/settings.json` that now lives in this repo) even though I think I prefer Claude's (I do have a working neovim Rust setup, and Copilot's only works with VS Code).

# Misaligned text

Seeing an alignment issue, I asked Copilot the following:

> How can I have the `mode_text` appear in the centre, not justified to the left? The Help box is centred correctly.

It replied with nonsense. The correct answer was already provided by rust-analyzer, but I couldn't find the bindings in VS Code to accept its suggestion. (I have the correct bindings in neovim).

# Testing

I [asked Claude](https://claude.ai/chat/53a72b05-57c6-4186-8381-69b991a82701) (third message) how to do snapshot testing.
The result fails to compile due to the lack of the `render` method, so I asked Copilot to fix it:

> My test contains `app.render`, which does not compile: no method named render found for struct `App`. How do I render it?

It suggested a diff, which I accepted; this was *terrifying*, because it used the LLM to apply the diff to completely rewrite the file character-by-character.
I will not press that button again without `git add`-ing first!
The result does appear to be correct, though.

## Extract `load_from_file`

I selected the definition of `load_from_file`, and told Copilot:

> Extract a `load_from_string` version of this, and have `load_from_file` be a wrapper around it.

The result Just Worked (except for an import issue).

# Nixifying

I didn't use LLMs for this at all; just standard Google etc.
Both Claude and Copilot were totally useless for this.

# Dividers between boxes

At this point we consistently have this kind of display:

```
      ┌────┐┌────┐┌────│┌────┐┌────┐┌────│┌────┐┌────┐┌────│
      │    ││    ││    ││ 2  ││ 6  ││    ││ 7  ││    ││ 1  │
      └────┘└────┘└────│└────┘└────┘└────│└────┘└────┘└────│
      ┌────┐┌────┐┌────│┌────┐┌────┐┌────│┌────┐┌────┐┌────│
      │ 6  ││ 8  ││    ││    ││ 7  ││    ││    ││ 9  ││    │
```

Notice the unsightly overlap between the dividing lines and the boxes.

Claude [was eventually able to fix this](https://claude.ai/chat/5664acec-e180-4057-ad4e-8ca3b560ac79), but it took a bunch of adjustments to my method of asking.

# Review

[Claude's review](https://claude.ai/chat/b9287623-9332-4dcd-adee-bf09a09fb8d7) at around commit fd8fcb80fb63fbd9555e5124a128325c2c189e94:

```
I'll analyze this code from a structural and idiomatic perspective. Here are the key areas for improvement:

Error Handling


The load_from_string method silently truncates input if it exceeds 9x9 dimensions. It should either return an Error variant or validate dimensions upfront.
The Result type from file operations is propagated but never handled meaningfully - errors should be wrapped in a domain-specific error type that provides context.


Type Safety


Cell coordinates are represented as raw usize values throughout. This creates potential for confusion between x/y coordinates. A newtype pattern like GridPosition { x: usize, y: usize } would be more type-safe.
InputMode could be more precisely modeled - the three modes represent fundamentally different operations on different aspects of cell state.


Data Structure Design


The grid field uses Vec<Vec<Cell>> which is suboptimal for a fixed-size 9x9 grid. A flat [Cell; 81] with coordinate translation functions would be more efficient and prevent invalid states.
SortedVec for corner/centre numbers seems overengineered - since we're dealing with single-digit numbers, a simple bitset would be more efficient.


State Management


The App struct combines UI state (cursor position, input mode) with game state (grid). These concerns should be separated.
Cell modification logic in handle_input has complex branching that could be simplified by making invalid states unrepresentable through types.


Code Organization


UI layout code is mixed with game logic. The ui function is doing too much - it should be split into smaller, focused components.
Drawing functions contain magic numbers (e.g., cell dimensions) that should be constants or configuration.


Rust Idioms


saturating_sub is used inconsistently - sometimes bounds are checked with min, other times with saturating_sub.
The Default derivation for Cell implies that an empty cell is the default state, but the domain logic suggests otherwise.


Test Coverage


The single snapshot test is insufficient. Property-based tests for grid operations and unit tests for state transitions would provide better coverage.
Test setup could be simplified with a builder pattern for App construction.


Documentation


Public types lack documentation comments explaining their invariants and usage.
Complex state transition logic (especially in handle_input) needs documentation explaining the business rules.


Memory Efficiency


Storing both corner and centre numbers as Vecs is wasteful given their small, fixed maximum sizes. A fixed-size array would be more appropriate.


Drawing Logic


The draw_cell function has boolean parameters for thick borders. This would be clearer as an enum describing the cell's position in the grid.
The rendering code assumes a fixed terminal size without graceful degradation for smaller terminals.

These issues don't prevent the program from functioning, but addressing them would improve maintainability, correctness, and performance. Would you like me to elaborate on any particular aspect?
```
