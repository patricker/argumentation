import React, {useState} from 'react';
import styles from './styles.module.css';

type Props = {
  name: string;
  premises: string[];
  conclusion: string;
  criticalQuestions?: string[];
};

export default function SchemeCard({name, premises, conclusion, criticalQuestions = []}: Props) {
  const [openCQ, setOpenCQ] = useState(false);
  return (
    <div className={styles.card}>
      <div className={styles.header}>{name}</div>
      <div className={styles.body}>
        <div className={styles.section}>
          <div className={styles.label}>Premises</div>
          <ul className={styles.list}>
            {premises.map((p, i) => <li key={i}>{p}</li>)}
          </ul>
        </div>
        <div className={styles.section}>
          <div className={styles.label}>Conclusion</div>
          <div className={styles.conclusion}>∴ {conclusion}</div>
        </div>
        {criticalQuestions.length > 0 && (
          <div className={styles.section}>
            <button className={styles.cqToggle} onClick={() => setOpenCQ(!openCQ)}>
              {openCQ ? '▾' : '▸'} Critical questions ({criticalQuestions.length})
            </button>
            {openCQ && (
              <ol className={styles.list}>
                {criticalQuestions.map((cq, i) => <li key={i}>{cq}</li>)}
              </ol>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
