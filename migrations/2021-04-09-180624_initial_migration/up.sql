CREATE TABLE dict (
  id SERIAL PRIMARY KEY,
  sequence INTEGER NOT NULL,
  reading TEXT NOT NULL,
  kanji boolean NOT NULL,
  no_kanji boolean NOT NULL,
  priorities TEXT[],
  information INTEGER[],
  kanji_info INTEGER[],
  jlpt_lvl INTEGER,
  is_main BOOLEAN NOT NULL
);
CREATE INDEX index_reading_dict on dict using pgroonga (reading);
CREATE INDEX index_reading_dict_text_pattern_ops on dict (reading text_pattern_ops);
CREATE INDEX index_seq_dict ON dict (sequence);

CREATE TABLE sense (
  id SERIAL PRIMARY KEY,
  sequence INTEGER NOT NULL,
  language INTEGER NOT NULL,
  gloss_pos INTEGER NOT NULL,
  gloss TEXT NOT NULL,
  misc TEXT,
  part_of_speech TEXT[],
  dialect TEXT,
  xref TEXT,
  gtype INTEGER,
  field TEXT,
  information TEXT,
  antonym TEXT,
  pos_simplified INTEGER[]
);
CREATE INDEX index_seq_sense ON sense (sequence);
CREATE INDEX index_gloss_sense on sense using pgroonga (gloss);
CREATE INDEX index_gloss_sense_text_pattern_ops on sense (gloss text_pattern_ops);
CREATE INDEX index_lang_sense ON sense (language);
CREATE INDEX index_pos_simple_sense ON sense (pos_simplified);

CREATE TABLE kanji (
  id SERIAL PRIMARY KEY,
  literal CHAR(1) NOT NULL,
  meaning TEXT[] NOT NULL,
  grade INTEGER,
  stroke_count INTEGER NOT NULL,
  frequency INTEGER,
  jlpt INTEGER,
  variant TEXT[],
  onyomi TEXT[],
  kunyomi TEXT[],
  chinese TEXT,
  korean_r TEXT[],
  korean_h TEXT[],
  natori TEXT[],
  kun_dicts INTEGER[]
);
CREATE INDEX index_literal_kanji ON kanji (literal);

CREATE TABLE name (
  id SERIAL PRIMARY KEY,
  sequence INTEGER NOT NULL,
  kana TEXT NOT NULL,
  kanji TEXT,
  transcription TEXT NOT NULL,
  name_type INTEGER[],
  xref TEXT
);
CREATE INDEX index_kana_name ON name (kana text_pattern_ops);
CREATE INDEX index_kanji_name ON name (kanji text_pattern_ops);
CREATE INDEX index_transcription_name ON name (transcription);

CREATE TABLE sentence (
  id SERIAL PRIMARY KEY,
  content TEXT NOT NULL,
  furigana TEXT NOT NULL
);
CREATE INDEX index_sentence_content ON sentence using pgroonga (content) WITH (tokenizer='TokenMecab');

CREATE TABLE sentence_translation (
  id SERIAL PRIMARY KEY,
  sentence_id INTEGER NOT NULL,
  language INTEGER NOT NULL,
  content TEXT NOT NULL, 
  foreign key (sentence_id) references sentence(id)
);
CREATE INDEX index_sentence_translation_content ON sentence_translation using pgroonga (content);
CREATE INDEX index_sentence_translation_language ON sentence_translation (language);

CREATE TABLE sentence_vocabulary (
  id SERIAL PRIMARY KEY,
  sentence_id INTEGER NOT NULL,
  dict_sequence INTEGER NOT NULL,
  start INTEGER NOT NULL,
  foreign key (sentence_id) references sentence(id)
);


CREATE OR REPLACE FUNCTION is_kanji(IN inp text)
 RETURNS boolean AS
 $BODY$
     SELECT
         inp ~ '^[\x3400-\x4DB5\x4E00-\x9FCB\xF900-\xFA6A]*$'
 $BODY$
 LANGUAGE sql
 IMMUTABLE
 STRICT;

CREATE OR REPLACE FUNCTION is_kana(IN inp text)
 RETURNS boolean AS
 $BODY$
     SELECT
         inp ~ '^[ぁ-んァ-ン]*$'
 $BODY$
 LANGUAGE sql
 IMMUTABLE
 STRICT;

CREATE OR REPLACE FUNCTION is_hiragana(IN inp text)
 RETURNS boolean AS
 $BODY$
     SELECT
         inp ~ '^[ぁ-ゔゞ゛゜ー]*$'
 $BODY$
 LANGUAGE sql
 IMMUTABLE
 STRICT;

CREATE OR REPLACE FUNCTION is_katakana(IN inp text)
 RETURNS boolean AS
 $BODY$
     SELECT
         inp ~ '^[ァ-・ヽヾ゛゜ー]*$'
 $BODY$
 LANGUAGE sql
 IMMUTABLE
 STRICT;

CREATE OR REPLACE FUNCTION ends_with_hiragana(IN inp text)
 RETURNS boolean AS
 $BODY$
     SELECT
         inp ~ '[ぁ-ゔゞ゛゜ー]+$'
 $BODY$
 LANGUAGE sql
 IMMUTABLE
 STRICT;

CREATE OR REPLACE FUNCTION get_kun_dicts(i integer)
RETURNS table (id INTEGER, sequence INTEGER, reading TEXT, kanji boolean, no_kanji boolean, priorities TEXT[], information INTEGER[], kanji_info INTEGER[], jlpt_lvl INTEGER, is_main boolean)  AS $$
  select * from dict where reading in ( SELECT literal || SUBSTRING(UNNEST(kunyomi) from POSITION('.' in  UNNEST(kunyomi))+1) from kanji where kanji.id = i)
$$
LANGUAGE sql stable;

CREATE OR REPLACE FUNCTION find_kanji_by_meaning(mea TEXT)
  RETURNS setof "kanji" AS $$
    select id, literal, meaning, grade, stroke_count, frequency, jlpt, variant, onyomi, kunyomi, chinese, korean_r, korean_h, natori, kun_dicts
  from (select *, unnest(meaning) m from kanji) x order by mea <-> m limit 4
  $$
 LANGUAGE sql stable;

CREATE OR REPLACE FUNCTION search_sentence_foreign(squery TEXT, off integer, lim integer, lang integer)
    RETURNS table (content text, furigana text, translation text, id integer) AS $$
      SELECT sentence.content, sentence.furigana, sentence_translation.content,sentence.id FROM sentence
       INNER JOIN sentence_translation ON sentence_translation.sentence_id = sentence.id
         WHERE language = lang
       ORDER BY squery <-> sentence_translation.content
       LIMIT lim OFFSET off
  $$
 LANGUAGE sql stable;

CREATE OR REPLACE FUNCTION search_sentence_jp(squery TEXT, off integer, lim integer, lang integer)
    RETURNS table (content text, furigana text, translation text,id integer) AS $$
      SELECT sentence.content, sentence.furigana, sentence_translation.content, sentence.id FROM sentence
       INNER JOIN sentence_translation ON sentence_translation.sentence_id = sentence.id
         WHERE language = lang and
           sentence.content like '%'||squery||'%'
         ORDER BY squery <-> sentence.content limit lim offset off
    $$
 LANGUAGE sql stable;
