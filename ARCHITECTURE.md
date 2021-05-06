# `VM`

The VM manages the backend of the VM. It initializes all `Sprite`s and runs them. It also handles a lot of the broadcast messages.

## `Global`

Contains the global state, such as variables, broadcast channel, and keyboard inputs.

### `Broadcaster`

Certain blocks and the VM subscribe to the `Broadcaster` to receive broadcast messages. Broadcast messages are not limited to those sent by event blocks. Blocks can use broadcast messages to tell the VM to modify other sprites, such as to clone a sprite.

## `Sprite`

Holds all threads and doesn't do much else.

### `SpriteRuntime`

Contains the sprite state. It is also responsible for drawing the sprite.

### `Thread`

Contains all blocks of the thread and manages the control flow.

- Substack blocks are stored in `HashMap<BlockID, Box<dyn Block>>`.
- A `Thread` has a loop stack to keep track of where to go back after a loop.

### `Block`

A `Block` is a trait implemented by all blocks. A block can have fields, inputs and substacks.

- Field: A constant string shown as a menu in the block editor 
- Input: An oval block that emits values. Cannot be used as a substack. Input blocks are owned by `Block`s.
- Substack: Blocks connected below another block and can be executed. Cannot be used as an input. After `execute()`, it returns the `BlockID` of the next block to execute.