# Not so hardware Text Editor

Once upon a time, a little Ecconia started to build a Typewriter in Logic World (LW).
Such a typewriter should serve the ability of writing assembly inside of LW.
But a typewriter is not enough, a text editor had to be made - entirely in hardware.

## Old features:

When the typewriter was created in LW, it only supported these features:

- Typing letters (wow)
- Deleting letters (backspace)
- Moving the cursor left and right (allows letters on both sides of the cursor)

Being able to move the cursor was the first step away from a simple typewriter.

## Core concept:

The text editor uses a custom charset of 7 bits (expandable - Non ASCII!).\
All written text is stored in a single memory of 4KB.\
The memory contains two stacks starting at the beginning and end of the memory. Each stack represents one side of the cursor. If the cursor is moved over a letter, the letter has to be transferred onto the other stack.

This program basically functions exactly the same way.

## Missing features:

To have a decent text editor some features are missing:

- Better cursor navigation: Up/Down & Page-Up/Down
- Advanced cursor navigation: Control+Left/Right (to skip words)
- Text selection & clipboard (gonna be fun in hardware)

# Contact:

To talk with me about this project, feel free to join the [Logic World Discord Server](https://discord.gg/C5Qkk53), you will find me there :)

There also exists a self-hosted Logic World server for this project. You will have to ask me for access.
