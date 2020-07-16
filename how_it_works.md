# Terminology

- thread: A graph of blocks; it consists of the main stack and branching stacks
- primitive: Any block or function
- block: One step in a thread; contains inputs and the runtime
- input: An argument or branch
- argument: Function inputs in a block; an argument is an upside-down tree
- hat: First block in a thread
- function: Takes 0 or more arguments and returns a value; cannot be directly added to a thread (In the editor it has a rounded or diamond shape)
- runtime: VM context
- monitor: Value display in the output
- variable: A function that holds a value and has no inputs
