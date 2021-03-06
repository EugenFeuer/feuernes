/*
https://wiki.nesdev.com/w/index.php/PPU_registers
    Common Name	Address	Bits	    Notes
    PPUCTRL	    $2000	VPHB SINN	NMI enable (V), PPU master/slave (P), sprite height (H), background tile select (B), sprite tile select (S), increment mode (I), nametable select (NN)
    PPUMASK	    $2001	BGRs bMmG	color emphasis (BGR), sprite enable (s), background enable (b), sprite left column enable (M), background left column enable (m), greyscale (G)
    PPUSTATUS	$2002	VSO- ----	vblank (V), sprite 0 hit (S), sprite overflow (O); read resets write pair for $2005/$2006
    OAMADDR	    $2003	aaaa aaaa	OAM read/write address
    OAMDATA	    $2004	dddd dddd	OAM data read/write
    PPUSCROLL	$2005	xxxx xxxx	fine scroll position (two writes: X scroll, Y scroll)
    PPUADDR	    $2006	aaaa aaaa	PPU read/write address (two writes: most significant byte, least significant byte)
    PPUDATA 	$2007	dddd dddd	PPU data read/write
    OAMDMA	    $4014	aaaa aaaa	OAM DMA high address
*/

pub mod address;
pub mod controller;
pub mod data;
pub mod mask;
pub mod oam_address;
pub mod oam_data;
pub mod scroll;
pub mod status;

pub trait BitwiseRegister {
    fn update_bits(&mut self, bits: u8) {}
    fn get_bits(&self) -> u8 {
        0
    }
}
