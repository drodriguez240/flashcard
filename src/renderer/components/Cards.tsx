import * as React from 'react';

import db, { Card } from '../model/Database';
import search from '../controller/Search';

import CardSelectable from './CardSelectable';
import Button from './Button';
import Dropdown, { DropdownItem } from './Dropdown';
import Input from './Input';
import Modal from './Modal';
import Empty from './Empty';

// TODO: Show amount of selected cards somewhere
// TODO: Fix bug where isAllSelected() returns true when switching between topics that have the same amount of cards
// TODO: Make sort button change icon based on selected sorting type?

enum CardSort {
    Newest,
    Oldest,
    RetentionRateAsc,
    RetentionRateDesc
}

const Cards = (props: ICardsProps) => {
    const [showBack, setShowBack] = React.useState<boolean>(false);
    const [showAmount, setShowAmount] = React.useState<number>(20);
    const [selected, setSelected] = React.useState<number>(props.cards.filter(c => c.selected).length);
    const [searchResults, setSearchResults] = React.useState<Card[]>(null);

    const [sortBy, setSortBy] = React.useState<CardSort>(CardSort.Newest);
    const [showBulkMove, setShowBulkMove] = React.useState<boolean>(false);

    const cards = (): Card[] => {
        return searchResults || props.cards;
    }

    const onSelect = () => {
        setSelected(s => s + 1);
    }

    const onDeselect = () => {
        setSelected(s => s - 1);
    }

    const onDelete = (card: Card) => {
        if (card.selected) onDeselect();
        updateCards();
    }

    const toggleSelectAll = () => {
        cards().forEach(c => c.selected = !isAllSelected());
        setSelected(props.cards.filter(c => c.selected).length);
    }

    const isAllSelected = (): boolean => {
        return selected === props.cards.length;
    }

    const onSearch = (query: string) => {
        if (query === "") onClearSearch();
        else setSearchResults(search.query(query, props.cards));
    }

    const onClearSearch = () => {
        setSearchResults(null);
    }

    const sort = (cards: Card[]): Card[] => {
        switch (sortBy) {
            case CardSort.Newest:
                return cards.sort((a, b) => b.id - a.id);
            case CardSort.Oldest:
                return cards.sort((a, b) => a.id - b.id);
            case CardSort.RetentionRateDesc:
                return cards.sort((a, b) => b.retentionRate() - a.retentionRate());
            case CardSort.RetentionRateAsc:
                return cards.sort((a, b) => a.retentionRate() - b.retentionRate());
            default:
                return cards;
        }
    }

    const updateCards = () => {
        props.onCardChange();
    }

    if (props.cards.length === 0) {
        return (
            <Empty icon="content_copy" message="No cards" />
        );
    }

    return (
        <div className="col col-center full-width">
            <h2>Cards</h2>

            <section>
                <Input
                    className="search-input"
                    placeholder="Search..."
                    onChange={onSearch}
                    onClear={onClearSearch}
                    icon="search"
                />
            </section>

            <section className="row row-center col-center wrap space-fixed">
                <Button
                    name="Show answer"
                    icon={showBack ? "check_box" : "check_box_outline_blank"}
                    action={() => setShowBack(show => !show)}
                />

                <Button
                    name="Select all"
                    icon={isAllSelected() ? "check_box" : "check_box_outline_blank"}
                    action={toggleSelectAll}
                />

                <Dropdown name="Sort" icon="sort" showDownArrow={true}>
                    <DropdownItem name="Newest" icon="arrow_upward" action={() => setSortBy(CardSort.Newest)} />
                    <DropdownItem name="Oldest" icon="arrow_downward" action={() => setSortBy(CardSort.Oldest)} />
                    <DropdownItem name="Retention Rate" icon="trending_down" action={() => setSortBy(CardSort.RetentionRateDesc)} />
                    <DropdownItem name="Retention Rate" icon="trending_up" action={() => setSortBy(CardSort.RetentionRateAsc)} />
                </Dropdown>

                <Dropdown name="Bulk" icon="assignment" showDownArrow={true}>
                    <DropdownItem name="Move" icon="arrow_forward" action={() => setShowBulkMove(true)} />
                </Dropdown>
            </section>

            <section className="cards row-center full-width">
                {sort(cards()).slice(0, showAmount).map(c =>
                    <CardSelectable
                        card={c}
                        showBack={showBack}
                        onDelete={onDelete}
                        onSelect={onSelect}
                        onDeselect={onDeselect}
                        key={c.id}
                    />
                )}
            </section>

            {showAmount < cards().length
                ?   <section className="row space-fixed">
                        <Button name="Load more" icon="cached" action={() => setShowAmount(amount => amount + 20)} />
                        <Button name="Load all" icon="done_all" action={() => setShowAmount(props.cards.length)} />
                    </section>
                :   null
            }

            <Modal show={showBulkMove} onClickOutside={() => setShowBulkMove(false)}>
                <h2>Move</h2>
                <p>Move selected cards to another topic.</p>
                <section>
                    <section className="col">
                        <label>Topics</label>
                        <select>
                            {db.topics.getAll().map(t => <option key={t.id} value={t.id}>{t.name}</option>)}
                        </select>
                    </section>
                    <section className="row space-between">
                        <Button name="Move" icon="done" action={() => console.log("TODO: bulk move cards...")} />
                        <Button name="Cancel" icon="close" action={() => setShowBulkMove(false)} />
                    </section>
                </section>
            </Modal>
        </div>
    );
}

interface ICardsProps {
    cards: Card[]
    onCardChange(): void
}

export default Cards;