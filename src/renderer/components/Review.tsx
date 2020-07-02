import * as React from 'react';
import { useParams } from "react-router-dom";

import db, { Card as CardEntity } from '../model/Database';
import KeyCodes from '../controller/KeyCodes';

import Card from './Card';
import Button from './Button';
import Empty from './Empty';

// TODO: custom study

const shuffle = (arr: CardEntity[]): CardEntity[] => {
    for (let currentIndex = arr.length - 1; currentIndex > 0; currentIndex--) {
        const newIndex = Math.floor(Math.random() * (currentIndex + 1));
        const temp = arr[currentIndex];
        arr[currentIndex] = arr[newIndex];
        arr[newIndex] = temp;
    }
    return arr;
}

const Review = () => {
    const { topicId } = useParams<{topicId: string}>();
    const id = Number(topicId);

    const [cards, setCards] = React.useState<CardEntity[]>(shuffle(id ? db.cards.getDue(id) : db.cards.getDue()));
    const [index, setIndex] = React.useState<number>(0);
    const [card, setCard] = React.useState<CardEntity>(cards[index]);
    const [showAnswer, setShowAnswer] = React.useState<boolean>(false);

    const [total, setTotal] = React.useState<number>(cards.length);
    
    React.useEffect(() => {
        if (index > cards.length - 1) {
            //shuffle(cards);
            setIndex(0);
        }
        showCard();
    }, [index, cards]);

    const handleReview = (success: boolean) => {
        card.review(success);
        setCards(oldCards => oldCards.filter(c => c.id !== card.id));
    }

    const skip = () => {
        setIndex(i => i + 1);
    }

    const showCard = () => {
        setShowAnswer(false);
        setCard(cards[index]);
    }

    const onDelete = () => {
        setTotal(t => t - 1);
        setCards(oldCards => oldCards.filter(c => c.id !== card.id));
    }

    if (db.cards.size() === 0) {
        return (
            <div className="content">
                <Empty icon="content_copy" message="No cards" />
            </div>
        );
    }

    if (card === undefined) {
        return (
            <div className="content">
                <Empty icon="mood" message="No cards to review" />
            </div>
        );
    }

    return (
        <div className="content col col-center space-between full-height">
            <section>
                <label>{total - cards.length} / {total}</label>
            </section>

            <Card
                card={card}
                showBack={showAnswer}
                onDelete={onDelete}
            />

            <div className="review-buttons space-fixed">
                {showAnswer || <Button icon="lock_open" action={() => setShowAnswer(true)} shortcut={KeyCodes.Space} />}
                {showAnswer && <Button icon="done" action={() => handleReview(true)} shortcut={KeyCodes.ArrowUp} />}
                {showAnswer && <Button icon="close" action={() => handleReview(false)} shortcut={KeyCodes.ArrowDown} />}
                <Button icon="double_arrow" action={skip} shortcut={KeyCodes.ArrowRight} />
            </div>
        </div>
    );
}

export default Review;