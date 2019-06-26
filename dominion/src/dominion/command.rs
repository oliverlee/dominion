use crate::dominion::card::CardKind;
use crate::dominion::types::{CardVec, Location};
use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub(crate) enum ParseCommandError {
    InvalidCommand,
    InvalidPlayerId,
    UnspecifiedPlayerId,
    UndefinedCardKind,
    UnspecifiedCardKind,
}

impl Error for ParseCommandError {
    fn description(&self) -> &str {
        match self {
            ParseCommandError::InvalidCommand => "failed to parse command",
            ParseCommandError::InvalidPlayerId => "failed to parse player id arg",
            ParseCommandError::UnspecifiedPlayerId => "no player id arg to parse",
            ParseCommandError::UndefinedCardKind => "failed to parse card arg",
            ParseCommandError::UnspecifiedCardKind => "no card arg to parse",
        }
    }
}

impl fmt::Display for ParseCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        "provided string was not a valid command".fmt(f)
    }
}

impl From<serde_json::error::Error> for ParseCommandError {
    fn from(error: serde_json::error::Error) -> Self {
        ParseCommandError::UndefinedCardKind
    }
}

impl From<std::num::ParseIntError> for ParseCommandError {
    fn from(error: std::num::ParseIntError) -> Self {
        ParseCommandError::InvalidPlayerId
    }
}

type CommandResult<T> = std::result::Result<T, ParseCommandError>;

#[derive(Debug, PartialEq)]
pub(crate) enum Command {
    View(Location),
    EndPhase,
    PlayCard(CardKind),
    BuyCard(CardKind),
    SelectCards(CardVec),
}

impl FromStr for Command {
    type Err = ParseCommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let args: Vec<_> = s.split_whitespace().collect();
        let (&command, args) = args
            .split_first()
            .ok_or(ParseCommandError::InvalidCommand)?;

        match command {
            "end" => Ok(Command::EndPhase),
            "play" => Ok(Command::PlayCard(
                args.get(0)
                    .ok_or(ParseCommandError::UnspecifiedCardKind)?
                    .parse()?,
            )),
            "buy" => Ok(Command::BuyCard(
                args.get(0)
                    .ok_or(ParseCommandError::UnspecifiedCardKind)?
                    .parse()?,
            )),
            "select" => {
                let cards = args
                    .into_iter()
                    .map(|s| s.parse::<CardKind>())
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Command::SelectCards(cards))
            }
            "supply" => Ok(Command::View(Location::Supply)),
            "hand" => Ok(Command::View(Location::Hand {
                player_id: args
                    .get(0)
                    .ok_or(ParseCommandError::UnspecifiedPlayerId)?
                    .parse()?,
            })),
            "discard" => Ok(Command::View(Location::Discard {
                player_id: args
                    .get(0)
                    .ok_or(ParseCommandError::UnspecifiedPlayerId)?
                    .parse()?,
            })),
            "play" => Ok(Command::View(Location::Play {
                player_id: args
                    .get(0)
                    .ok_or(ParseCommandError::UnspecifiedPlayerId)?
                    .parse()?,
            })),
            _ => Err(ParseCommandError::InvalidCommand),
        }
    }
}

pub fn help() -> &'static str {
    "Valid commands:\n\
     hand <i> - view player <i>'s hand\n\
     discard <i>- view player <i>'s discard pile\n\
     play <i> - view player <i>'s play zone\n\
     supply - view the game's supply\n\
     end - ends the current phase (action or buy)\n\
     play <card>\n\
     buy <card>"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominion::card::CardKind;
    use crate::dominion::types::Location;

    #[test]
    fn parse_view_hand_command() {
        assert_eq!(
            "hand 0".parse::<Command>().unwrap(),
            Command::View(Location::Hand { player_id: 0 })
        );
        assert_eq!(
            "hand 1".parse::<Command>().unwrap(),
            Command::View(Location::Hand { player_id: 1 })
        );
    }

    #[test]
    fn parse_view_hand_command_invalid_player_id() {
        assert_eq!(
            "hand -1".parse::<Command>().unwrap_err(),
            ParseCommandError::InvalidPlayerId
        );
    }

    #[test]
    fn parse_view_supply_command() {
        assert_eq!(
            "supply".parse::<Command>().unwrap(),
            Command::View(Location::Supply)
        );
    }

    #[test]
    fn parse_end_phase_command() {
        assert_eq!("end".parse::<Command>().unwrap(), Command::EndPhase);
    }

    #[test]
    fn parse_play_card_command() {
        assert_eq!(
            "play Copper".parse::<Command>().unwrap(),
            Command::PlayCard(CardKind::Copper)
        );
        assert_eq!(
            "play ThroneRoom".parse::<Command>().unwrap(),
            Command::PlayCard(CardKind::ThroneRoom)
        );
    }

    #[test]
    fn parse_buy_card_command() {
        assert_eq!(
            "buy Gold".parse::<Command>().unwrap(),
            Command::BuyCard(CardKind::Gold)
        );
    }

    #[test]
    fn parse_buy_card_command_undefined_card_kind() {
        assert_eq!(
            "buy Platinum".parse::<Command>().unwrap_err(),
            ParseCommandError::UndefinedCardKind
        );
    }

    #[test]
    fn parse_buy_card_command_unspecified_card_kind() {
        assert_eq!(
            "buy ".parse::<Command>().unwrap_err(),
            ParseCommandError::UnspecifiedCardKind
        );
    }

    #[test]
    fn parse_select_cards_command() {
        assert_eq!(
            "select ".parse::<Command>().unwrap(),
            Command::SelectCards(vec![])
        );
        assert_eq!(
            "select Gold Silver Copper Copper"
                .parse::<Command>()
                .unwrap(),
            Command::SelectCards(vec![
                CardKind::Gold,
                CardKind::Silver,
                CardKind::Copper,
                CardKind::Copper
            ])
        );
    }
}
