/**
 * Mobile Navigation Menu Handler
 * 
 * This script adds functionality to the hamburger menu for mobile devices.
 * It toggles the navigation menu and handles the hamburger icon animation.
 */
document.addEventListener('DOMContentLoaded', function() {
    // Get the hamburger button and navigation menu
    const hamburger = document.querySelector('.hamburger');
    const navMenu = document.getElementById('header-nav-ul');
    
    // Check if elements exist
    if (hamburger && navMenu) {
        // Add click event to hamburger button
        hamburger.addEventListener('click', function() {
            // Toggle active class on hamburger
            this.classList.toggle('active');
            
            // Toggle active class on navigation menu
            navMenu.classList.toggle('active');
            
            // Prevent scrolling when menu is open
            document.body.classList.toggle('menu-open');
        });
        
        // Close menu when clicking on a navigation link
        const navLinks = document.querySelectorAll('.header-nav-ul-li-a');
        navLinks.forEach(link => {
            link.addEventListener('click', function() {
                hamburger.classList.remove('active');
                navMenu.classList.remove('active');
                document.body.classList.remove('menu-open');
            });
        });
    }
    
    // Close menu when clicking outside
    document.addEventListener('click', function(event) {
        if (navMenu.classList.contains('active') && 
            !navMenu.contains(event.target) && 
            !hamburger.contains(event.target)) {
            hamburger.classList.remove('active');
            navMenu.classList.remove('active');
            document.body.classList.remove('menu-open');
        }
    });
});