:objects shaker1 - shaker
:objects left right - hand
:objects shot1 - shot
:objects ingredient1 - ingredient
:objects cocktail1 - cocktail
:objects dispenser1 - dispenser
:objects l0 l1 l2 - level

:init ontable shaker1
:init ontable shot1
:init dispenses dispenser1 ingredient1
:init clean shaker1
:init clean shot1
:init empty shaker1
:init empty shot1
:init handempty left
:init handempty right
:init shaker_empty_level shaker1 l0
:init shaker_level shaker1 l0
:init next l0 l1
:init next l1 l2
:init cocktail_part1 cocktail1 ingredient1
:init cocktail_part2 cocktail1 ingredient1

:goal contains shot1 cocktail1