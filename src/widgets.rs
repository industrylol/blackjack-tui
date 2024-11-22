use std::{
    fmt::{Display, Write as _},
    iter::zip,
    marker::PhantomData,
};

use rand::prelude::{thread_rng, SliceRandom};
use ratatui::{
    prelude::*,
    widgets::{Block, List, Widget, WidgetRef},
};

#[derive(Debug)]
pub struct Deck(Vec<Card>);
impl Deck {
    pub fn new() -> Self {
        let mut deck = Deck(NEW_DECK.to_vec());
        deck.shuffle(1);
        deck
    }

    pub fn new_hand<T>(&mut self) -> Hand<T> {
        Hand::new([self.draw(), self.draw()])
    }

    fn draw(&mut self) -> Card {
        if let Some(card) = self.0.pop() {
            card
        } else {
            *self = Deck::new();
            self.0.pop().unwrap()
        }
    }

    pub fn shuffle(&mut self, num: u8) {
        let mut rng = thread_rng();
        for _ in 0..num {
            self.0.shuffle(&mut rng);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Player;
#[derive(Clone, Copy, Debug)]
pub struct Dealer;

#[derive(Clone, Copy, Debug)]
enum HandStatus {
    Active,
    Hold,
    Revealed,
}

#[derive(Clone, Copy, Debug)]
enum HandOwner {
    Player,
    Dealer,
}
impl Display for HandOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let owner = match self {
            HandOwner::Player => "Player",
            HandOwner::Dealer => "Dealer",
        };
        write!(f, "{}", owner)
    }
}

#[derive(Debug)]
pub struct Hand<T>(Vec<Card>, HandStatus, PhantomData<T>);
impl<T> Hand<T> {
    fn new(initial: [Card; 2]) -> Self {
        Self(initial.to_vec(), HandStatus::Active, PhantomData)
    }

    pub fn count_value(&self) -> u8 {
        // sum all non-aces
        let val = self
            .0
            .iter()
            .filter(|Card(kind, _)| !matches!(kind, Rank::Ace))
            .fold(0, |acc, Card(kind, _)| acc + kind.get_value());

        // determine ace values based on existing sum
        self.0
            .iter()
            .filter(|Card(kind, _)| matches!(kind, Rank::Ace))
            .fold(val, |acc, Card(kind, _)| {
                if (acc + kind.get_value()) > 21 {
                    acc + 1
                } else {
                    acc + kind.get_value()
                }
            })
    }

    pub fn is_bust(&self) -> bool {
        self.count_value() > 21
    }

    pub fn hit(&mut self, deck: &mut Deck) {
        self.0.push(deck.draw());
    }

    pub fn is_active(&self) -> bool {
        matches!(self.1, HandStatus::Active)
    }

    pub fn hold(&mut self) {
        self.1 = HandStatus::Hold
    }

    fn render_hand(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        owner: HandOwner,
    ) where
        Self: Sized,
    {
        let constraints = Constraint::from_lengths((0..6).map(|_| Card::WIDTH).collect::<Vec<_>>());

        let mut block = Block::bordered().title(owner.to_string());
        if matches!(owner, HandOwner::Player) {
            block = block
                .title_bottom(Line::from("1) Hit").left_aligned())
                .title_bottom(Line::from("2) Hold").centered())
                .title_bottom(Line::from("Q) Quit").right_aligned());
        }

        let inner_area = block.inner(area);
        block.render(area, buf);

        let [card_area, status_area] =
            Layout::vertical([Constraint::Percentage(85), Constraint::Fill(1)])
                .spacing(1)
                .areas::<2>(inner_area);

        let [card_top_area, card_bottom_area] =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(card_area);

        let card_top_row = Layout::horizontal(constraints.clone())
            .flex(ratatui::layout::Flex::Legacy)
            .spacing(2)
            .areas::<6>(card_top_area);

        let card_bottom_row = Layout::horizontal(constraints)
            .flex(ratatui::layout::Flex::Legacy)
            .spacing(2)
            .areas::<6>(card_bottom_area);

        // render cards
        for (index, card) in self.0.iter().enumerate() {
            let layout_rect = if index < 6 {
                card_top_row[index]
            } else {
                card_bottom_row[index - 6]
            };
            if matches!(owner, HandOwner::Dealer)
                && !matches!(self.1, HandStatus::Revealed)
                && index == 0
            {
                FaceDownCard::render(FaceDownCard, layout_rect, buf);
            } else {
                card.render(layout_rect, buf);
            }
        }

        // render hand status
        if matches!(owner, HandOwner::Dealer) {
            Widget::render(
                List::new([format!("Status: {:?}", self.1)]),
                status_area,
                buf,
            );
        } else {
            Widget::render(
                List::new([
                    format!("Status: {:?}", self.1),
                    format!("Value: {}", self.count_value()),
                ]),
                status_area,
                buf,
            );
        }
    }
}
impl Hand<Dealer> {
    pub fn do_dealer_action(&mut self, deck: &mut Deck) {
        if self.count_value() < 16 {
            self.hit(deck);
        } else {
            self.hold();
        }
    }

    pub fn reveal(&mut self) {
        self.1 = HandStatus::Revealed;
    }
}
impl<T> Display for Hand<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hand: ")?;
        for card in &self.0 {
            write!(f, "{card}, ")?;
        }

        let value = self.count_value();

        write!(f, "\nValue: {value}",)
    }
}
impl Widget for Hand<Player> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.render_ref(area, buf);
    }
}
impl Widget for Hand<Dealer> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.render_ref(area, buf);
    }
}
impl WidgetRef for Hand<Dealer> {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.render_hand(area, buf, HandOwner::Dealer);
    }
}
impl WidgetRef for Hand<Player> {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.render_hand(area, buf, HandOwner::Player);
    }
}

#[derive(Clone, Copy, Debug)]
struct Card(Rank, Suit);
impl Card {
    const WIDTH: u16 = 11;
}
impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Card(kind, suit) = self;
        write!(f, " {suit} {kind:?} ")
    }
}
impl Widget for Card {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let Card(rank, suit) = self;
        let mut card = String::new();
        let color = suit.color();
        let _ = writeln!(card, "╭─────────╮");
        let _ = writeln!(card, "|{:<9}|", format!("{}{}", suit, rank.get_rank()));
        let _ = writeln!(card, "|         |");
        let _ = writeln!(card, "|{:^9}|", format!("{}", rank));
        let _ = writeln!(card, "|         |");
        let _ = writeln!(card, "|{:>9}|", format!("{}{}", rank.get_rank(), suit));
        let _ = writeln!(card, "╰─────────╯");

        for (line, row) in zip(card.lines(), area.rows()) {
            let span = line.fg(color).bg(Color::White);
            span.render(row, buf);
        }
    }
}

struct FaceDownCard;
impl Widget for FaceDownCard {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let mut card = String::new();
        let _ = writeln!(card, "╭─────────╮");
        let _ = writeln!(card, "|{:x<9}|", "");
        let _ = writeln!(card, "|{:x<9}|", "");
        let _ = writeln!(card, "|{:x^9}|", "");
        let _ = writeln!(card, "|{:x<9}|", "");
        let _ = writeln!(card, "|{:x>9}|", "");
        let _ = writeln!(card, "╰─────────╯");

        for (line, row) in zip(card.lines(), area.rows()) {
            let span = line.fg(Color::Blue).bg(Color::White);
            span.render(row, buf);
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}
impl Rank {
    pub const fn get_value(&self) -> u8 {
        match self {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 10,
            Rank::Queen => 10,
            Rank::King => 10,
            Rank::Ace => 11,
        }
    }

    pub const fn get_rank(&self) -> &str {
        match self {
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
            Rank::Ace => "A",
        }
    }
}
impl Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // defer to debug impl
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug)]
enum Suit {
    Spade,
    Club,
    Diamond,
    Heart,
}
impl Suit {
    pub fn color(&self) -> Color {
        match self {
            Suit::Spade => Color::Black,
            Suit::Club => Color::Black,
            Suit::Diamond => Color::Red,
            Suit::Heart => Color::Red,
        }
    }
}
impl Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            Suit::Spade => "♠",
            Suit::Club => "♣",
            Suit::Diamond => "♦",
            Suit::Heart => "♥",
        };

        write!(f, "{symbol}")
    }
}

const NEW_DECK: [Card; 52] = [
    // spades
    Card(Rank::Two, Suit::Spade),
    Card(Rank::Three, Suit::Spade),
    Card(Rank::Four, Suit::Spade),
    Card(Rank::Five, Suit::Spade),
    Card(Rank::Six, Suit::Spade),
    Card(Rank::Seven, Suit::Spade),
    Card(Rank::Eight, Suit::Spade),
    Card(Rank::Nine, Suit::Spade),
    Card(Rank::Ten, Suit::Spade),
    Card(Rank::Jack, Suit::Spade),
    Card(Rank::Queen, Suit::Spade),
    Card(Rank::King, Suit::Spade),
    Card(Rank::Ace, Suit::Spade),
    // clubs
    Card(Rank::Two, Suit::Club),
    Card(Rank::Three, Suit::Club),
    Card(Rank::Four, Suit::Club),
    Card(Rank::Five, Suit::Club),
    Card(Rank::Six, Suit::Club),
    Card(Rank::Seven, Suit::Club),
    Card(Rank::Eight, Suit::Club),
    Card(Rank::Nine, Suit::Club),
    Card(Rank::Ten, Suit::Club),
    Card(Rank::Jack, Suit::Club),
    Card(Rank::Queen, Suit::Club),
    Card(Rank::King, Suit::Club),
    Card(Rank::Ace, Suit::Club),
    // diamonds
    Card(Rank::Two, Suit::Diamond),
    Card(Rank::Three, Suit::Diamond),
    Card(Rank::Four, Suit::Diamond),
    Card(Rank::Five, Suit::Diamond),
    Card(Rank::Six, Suit::Diamond),
    Card(Rank::Seven, Suit::Diamond),
    Card(Rank::Eight, Suit::Diamond),
    Card(Rank::Nine, Suit::Diamond),
    Card(Rank::Ten, Suit::Diamond),
    Card(Rank::Jack, Suit::Diamond),
    Card(Rank::Queen, Suit::Diamond),
    Card(Rank::King, Suit::Diamond),
    Card(Rank::Ace, Suit::Diamond),
    // hearts
    Card(Rank::Two, Suit::Heart),
    Card(Rank::Three, Suit::Heart),
    Card(Rank::Four, Suit::Heart),
    Card(Rank::Five, Suit::Heart),
    Card(Rank::Six, Suit::Heart),
    Card(Rank::Seven, Suit::Heart),
    Card(Rank::Eight, Suit::Heart),
    Card(Rank::Nine, Suit::Heart),
    Card(Rank::Ten, Suit::Heart),
    Card(Rank::Jack, Suit::Heart),
    Card(Rank::Queen, Suit::Heart),
    Card(Rank::King, Suit::Heart),
    Card(Rank::Ace, Suit::Heart),
];
