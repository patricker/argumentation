import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  description: React.JSX.Element;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Inspectable scene traces',
    description: (
      <>
        Each beat records the arguments asserted, the attacks that bound,
        and the residual that produced the acceptance — something you can
        read, replay, and reason about.
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
    title: 'β — scene intensity as a single parameter',
    description: (
      <>
        Tune how strictly attacks bind. Low β — every counter bites
        (courtroom energy). High β — counters slide off (boardroom
        cordiality). One parameter, a wide range of scene registers.
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
    title: 'Based on published research',
    description: (
      <>
        Primitives from Dung (1995), Walton-Reed-Macagno (2008), Cayrol &
        Lagasquie-Schiex (2005), Dunne et al. (2011), Modgil-Prakken ASPIC+
        (2014). Every type traces back to a paper we link.
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
