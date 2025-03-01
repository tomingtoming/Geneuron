use macroquad::prelude::*;
use nalgebra as na;
use ::std::f32::consts::PI;
use ::rand::Rng;

/// Camera effects handling (shake, pulse, etc)
pub struct CameraEffects {
    // Shake effect parameters
    pub shake_duration: f32,
    pub shake_intensity: f32,
    pub shake_decay: f32,
    pub shake_offset: na::Vector2<f32>,
    
    // Flash effect parameters
    pub flash_duration: f32,
    pub flash_color: Color,
    pub flash_alpha: f32,
    
    // Pulse effect (zoom pulse)
    pub pulse_duration: f32,
    pub pulse_intensity: f32,
    pub pulse_current: f32,
}

impl CameraEffects {
    /// Create new camera effects
    pub fn new() -> Self {
        CameraEffects {
            shake_duration: 0.0,
            shake_intensity: 0.0,
            shake_decay: 5.0,
            shake_offset: na::Vector2::new(0.0, 0.0),
            flash_duration: 0.0,
            flash_color: WHITE,
            flash_alpha: 0.0,
            pulse_duration: 0.0,
            pulse_intensity: 0.0,
            pulse_current: 0.0,
        }
    }
    
    /// Start a camera shake effect
    pub fn shake(&mut self, intensity: f32, duration: f32) {
        self.shake_intensity = intensity;
        self.shake_duration = duration;
    }
    
    /// Start a screen flash effect
    pub fn flash(&mut self, color: Color, duration: f32) {
        self.flash_color = color;
        self.flash_duration = duration;
        self.flash_alpha = 0.7; // Start with 70% opacity
    }
    
    /// Start a zoom pulse effect
    pub fn pulse(&mut self, intensity: f32, duration: f32) {
        self.pulse_intensity = intensity;
        self.pulse_duration = duration;
        self.pulse_current = 0.0;
    }
    
    /// Update all camera effects
    pub fn update(&mut self, dt: f32, screen_width: f32) {
        // Update shake effect
        if self.shake_duration > 0.0 {
            // Decrease shake duration
            self.shake_duration -= dt;
            
            // Calculate shake intensity with decay
            let current_intensity = if self.shake_duration <= 0.0 {
                self.shake_intensity = 0.0;
                self.shake_duration = 0.0;
                0.0
            } else {
                self.shake_intensity * (self.shake_duration.min(0.5) / 0.5)
            };
            
            // Generate random shake offset
            let angle = ::rand::random::<f32>() * PI * 2.0;
            let distance = current_intensity * ::rand::random::<f32>() * screen_width * 0.02;
            
            self.shake_offset.x = angle.cos() * distance;
            self.shake_offset.y = angle.sin() * distance;
        } else {
            self.shake_offset.x = 0.0;
            self.shake_offset.y = 0.0;
        }
        
        // Update flash effect
        if self.flash_duration > 0.0 {
            self.flash_duration -= dt;
            self.flash_alpha = if self.flash_duration <= 0.0 {
                self.flash_duration = 0.0;
                0.0
            } else {
                (self.flash_duration * 0.7).min(0.7) // Gradually fade out
            };
        }
        
        // Update pulse effect
        if self.pulse_duration > 0.0 {
            self.pulse_duration -= dt;
            if self.pulse_duration <= 0.0 {
                self.pulse_duration = 0.0;
                self.pulse_current = 0.0;
            } else {
                // Sinusoidal pulse
                let phase = (1.0 - self.pulse_duration / 1.0) * PI;
                self.pulse_current = self.pulse_intensity * phase.sin() * 0.5;
            }
        }
    }
    
    /// Draw flash effect overlay
    pub fn draw_effects(&self, viewport_width: f32, viewport_height: f32) {
        // Draw flash effect
        if self.flash_alpha > 0.0 {
            let mut flash_color = self.flash_color;
            flash_color.a = self.flash_alpha;
            draw_rectangle(0.0, 0.0, viewport_width, viewport_height, flash_color);
        }
    }
    
    /// Get current zoom pulse adjustment
    pub fn get_zoom_pulse(&self) -> f32 {
        1.0 + self.pulse_current
    }
    
    /// Get current shake offset
    pub fn get_shake_offset(&self) -> na::Vector2<f32> {
        self.shake_offset
    }
}
