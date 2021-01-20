:objects shaker1 - shaker
:objects left right - hand
:objects shot1 shot2 shot3 shot4 - shot
:objects ingredient1 ingredient2 - ingredient
:objects cocktail1 cocktail2 cocktail3 cocktail4 - cocktail
:objects dispenser1 dispenser2 - dispenser
:objects l0 l1 l2 - level

:init ontable shaker1
:init ontable shot1
:init ontable shot2
:init ontable shot3
:init ontable shot4
:init dispenses dispenser1 ingredient1
:init dispenses dispenser2 ingredient2
:init clean shaker1
:init clean shot1
:init clean shot2
:init clean shot3
:init clean shot4
:init empty shaker1
:init empty shot1
:init empty shot2
:init empty shot3
:init empty shot4
:init handempty left
:init handempty right
:init shaker_empty_level shaker1 l0
:init shaker_level shaker1 l0
:init next l0 l1
:init next l1 l2
:init cocktail_part1 cocktail1 ingredient1
:init cocktail_part2 cocktail1 ingredient2
:init cocktail_part1 cocktail2 ingredient1
:init cocktail_part2 cocktail2 ingredient2
:init cocktail_part1 cocktail3 ingredient1
:init cocktail_part2 cocktail3 ingredient2
:init cocktail_part1 cocktail4 ingredient1
:init cocktail_part2 cocktail4 ingredient2

:goal contains shot1 cocktail1
:goal contains shot2 cocktail2
:goal contains shot3 cocktail3
:goal contains shot4 cocktail4