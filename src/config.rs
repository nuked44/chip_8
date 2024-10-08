//  Do not change
pub const MEMSIZE: usize = 4096;
pub const FONT_POS_START: usize = 0x50;
pub const PROG_POS_START: u16 = 0x200;
pub const SCREEN_WIDTH: u8 = 64;
pub const SCREEN_HEIGHT: u8 = 32;

// Can change
pub const SCREEN_REFRESH_RATE: usize = 60; // 0>FPS>256
pub const INSTRUCTION_FREQUENCY: usize = 500; // 0>IPS>n

// --- Keys ---

// Keys for querty keyboard
pub const KEY_1: char = '1';
pub const KEY_2: char = '2';
pub const KEY_3: char = '3';
pub const KEY_4: char = 'q';
pub const KEY_5: char = 'w';
pub const KEY_6: char = 'e';
pub const KEY_7: char = 'a';
pub const KEY_8: char = 's';
pub const KEY_9: char = 'd';
pub const KEY_0: char = 'x';
pub const KEY_A: char = 'z';
pub const KEY_B: char = 'c';
pub const KEY_C: char = '4';
pub const KEY_D: char = 'r';
pub const KEY_E: char = 'f';
pub const KEY_F: char = 'v';

/* Keys for hex keyboard
pub const KEY_1: char = '1';
pub const KEY_2: char = '2';
pub const KEY_3: char = '3';
pub const KEY_4: char = '4';
pub const KEY_5: char = '5';
pub const KEY_6: char = '6';
pub const KEY_7: char = '7';
pub const KEY_8: char = '8';
pub const KEY_9: char = '9';
pub const KEY_0: char = '0';
pub const KEY_A: char = 'a';
pub const KEY_B: char = 'b';
pub const KEY_C: char = 'c';
pub const KEY_D: char = 'd';
pub const KEY_E: char = 'e';
pub const KEY_F: char = 'f';
*/
