import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  description: React.JSX.Element;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Auditable scene AI',
    description: (
      <>
        Every beat has a receipt: which arguments fired, which attacks bound,
        which residual produced the acceptance. No black-box LLM hallucinations.
      </>
    ),
  },
  {
    title: '60+ Walton schemes out of the box',
    description: (
      <>
        Argument from expert opinion, analogy, cause-to-effect, slippery slope —
        all as composable scheme instances with premises, conclusions, and critical
        questions.
      </>
    ),
  },
  {
    title: 'β — scene intensity as a first-class dial',
    description: (
      <>
        Tune how strictly attacks bind. Low β = every counter bites (courtroom
        energy); high β = counters slide off (boardroom cordiality). One knob,
        radically different scenes.
      </>
    ),
  },
  {
    title: 'Trait-inverted encounter bridge',
    description: (
      <>
        <code>StateActionScorer</code> and <code>StateAcceptanceEval</code> plug
        into any consumer's scene engine through the <code>encounter</code> crate's
        trait-inverted interface.
      </>
    ),
  },
  {
    title: 'Grounded in the canon',
    description: (
      <>
        Implements Dung (1995), Walton-Reed-Macagno (2008), Cayrol &
        Lagasquie-Schiex (2005), Dunne et al. (2011), Modgil-Prakken ASPIC+ (2014).
        Every primitive traces back to a paper.
      </>
    ),
  },
  {
    title: 'Rust-first, WASM-ready (soon)',
    description: (
      <>
        Workspace of small, composable crates. Zero unsafe. Deterministic by
        default. A WASM build is on the roadmap for browser-native demos.
      </>
    ),
  },
];

function Feature({title, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): React.JSX.Element {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
