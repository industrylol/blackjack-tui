use std::{cell::RefCell, cmp::Ordering, rc::Rc};

use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        style::Stylize,
    },
    layout::Flex,
    prelude::{Constraint, Frame, Layout, Line},
    widgets::{Block, Clear, List, ListItem},
};

mod widgets;
use widgets::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_as_tui()?;
    Ok(())
}

fn run_as_tui() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let mut game_state = GameState::PlayingHand;
    let mut deck = Deck::new();
    let player_hand = Rc::new(RefCell::new(deck.new_hand::<Player>()));
    let dealer_hand = Rc::new(RefCell::new(deck.new_hand::<Dealer>()));
    let deck = Rc::new(RefCell::new(deck));

    loop {
        let _player_hand = player_hand.clone();
        let _dealer_hand = dealer_hand.clone();
        let deck = deck.clone();
        terminal.draw(move |frame: &mut Frame| {
            use Constraint::{Fill, Length, Min};

            let vertical = Layout::vertical([Length(2), Min(0)]);
            let [title_area, main_area] = vertical.areas(frame.area());
            let horizontal = Layout::horizontal([Fill(1); 2]);
            let [left_area, right_area] = horizontal.areas(main_area);

            frame.render_widget(Block::bordered().title("Blackjack"), title_area);
            frame.render_widget(&*_player_hand.borrow(), left_area);
            frame.render_widget(&*_dealer_hand.borrow(), right_area);

            match game_state {
                GameState::PlayingHand => (),
                GameState::HandScoreScreen(hand_result) => {
                    let player_hand = _player_hand.borrow();
                    let dealer_hand = _dealer_hand.borrow();

                    let frame_area = frame.area();
                    let block = Block::bordered()
                        .title("Hand Result")
                        .title_bottom(Line::from("Any) New Hand").left_aligned())
                        .title_bottom(Line::from("q) Quit").right_aligned());
                    let vertical =
                        Layout::vertical([Constraint::Percentage(20)]).flex(Flex::Center);
                    let horizontal =
                        Layout::horizontal([Constraint::Percentage(40)]).flex(Flex::Center);
                    let [area] = vertical.areas(frame_area);
                    let [area] = horizontal.areas(area);

                    frame.render_widget(Clear, area);

                    let list_items: [ListItem; 2] = [
                        Line::from(
                            match hand_result {
                                HandResult::PlayerWin => format!("{hand_result:?}").green(),
                                HandResult::DealerWin => format!("{hand_result:?}").red(),
                                HandResult::Push => format!("{hand_result:?}").yellow(),
                                HandResult::Bust => format!("{hand_result:?}").red(),
                            }
                            .to_string(),
                        )
                        .into(),
                        Line::from(format!(
                            "You: {} Dealer: {}",
                            player_hand.count_value(),
                            dealer_hand.count_value()
                        ))
                        .into(),
                    ];

                    frame.render_widget(List::new(list_items).block(block), area);
                }
            }
        })?;

        if let Event::Key(key) = event::read()? {
            let mut player_hand = player_hand.borrow_mut();
            let mut dealer_hand = dealer_hand.borrow_mut();
            let mut deck = deck.borrow_mut();
            if matches!(key.kind, KeyEventKind::Release) {
                match game_state {
                    GameState::PlayingHand => match key.code {
                        KeyCode::Char(c) => match c {
                            '1' => {
                                player_hand.hit(&mut deck);
                                dealer_hand.do_dealer_action(&mut deck);
                                check_hand(&player_hand, &mut dealer_hand, &mut game_state);
                            }
                            '2' => {
                                player_hand.hold();
                                while dealer_hand.is_active() && !dealer_hand.is_bust() {
                                    dealer_hand.do_dealer_action(&mut deck);
                                    check_hand(&player_hand, &mut dealer_hand, &mut game_state);
                                }
                                check_hand(&player_hand, &mut dealer_hand, &mut game_state);
                            }
                            'q' => break,
                            _ => (),
                        },
                        KeyCode::Esc => break,
                        _ => (),
                    },
                    GameState::HandScoreScreen(_) => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        _ => {
                            *player_hand = deck.new_hand::<Player>();
                            *dealer_hand = deck.new_hand::<Dealer>();
                            game_state = GameState::PlayingHand;
                        }
                    },
                }
            }
        }
    }
    ratatui::restore();
    Ok(())
}

fn check_hand(
    player_hand: &Hand<Player>,
    dealer_hand: &mut Hand<Dealer>,
    game_state: &mut GameState,
) {
    if !player_hand.is_bust() && !player_hand.is_active() && !dealer_hand.is_active() {
        let player_value = player_hand.count_value();
        let dealer_value = dealer_hand.count_value();
        *game_state = GameState::HandScoreScreen(match player_value.cmp(&dealer_value) {
            Ordering::Less => HandResult::DealerWin,
            Ordering::Equal => HandResult::Push,
            Ordering::Greater => HandResult::PlayerWin,
        });
    } else if player_hand.is_bust() {
        *game_state = GameState::HandScoreScreen(HandResult::Bust);
    } else if dealer_hand.is_bust() {
        *game_state = GameState::HandScoreScreen(HandResult::PlayerWin);
    }

    if matches!(game_state, GameState::HandScoreScreen(_)) {
        dealer_hand.reveal();
    }
}

#[derive(Clone, Copy, Debug)]
enum GameState {
    PlayingHand,
    HandScoreScreen(HandResult),
}

#[derive(Clone, Copy, Debug)]
enum HandResult {
    PlayerWin,
    DealerWin,
    Push,
    Bust,
}
