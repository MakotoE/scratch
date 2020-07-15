# Terminology

- thread: A directed acyclic graph of blocks
- primitive: Any block or function
- block: One step in a thread; contains arguments and the runtime
- argument: Function inputs in a block; is a vector of upside-down trees
- hat: First block in a thread
- function: Takes 0 or more arguments and returns a value; cannot be directly added to a thread (In the editor it has a rounded or diamond shape)
- branch: The second edge in a block, such as in an "if" block
- runtime: VM context
- monitor: Value display in the output
- variable: A function that holds a value and has no inputs
