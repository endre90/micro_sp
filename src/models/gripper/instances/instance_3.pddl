:objects rooma roomb - room
:objects ball3 ball2 ball1 - ball
:objects left right - gripper

:init at-robby rooma
:init free left
:init free right
:init at ball3 rooma
:init at ball2 rooma
:init at ball1 rooma

:goal at ball3 roomb
:goal at ball2 roomb
:goal at ball1 roomb