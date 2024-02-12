import React, {useEffect, useMemo, useState} from 'react';
import clsx from 'clsx';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import BrowserOnly from '@docusaurus/BrowserOnly';
import useIsBrowser from '@docusaurus/useIsBrowser';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import styles from './index.module.css';
import {
    Md3DRotation,
    MdArrowRight, MdArrowRightAlt,
    MdBuild,
    MdClose,
    MdDeviceHub,
    MdGradient,
    MdHomeRepairService,
    MdRefresh,
    MdReplay,
    MdViewInAr
} from "react-icons/md";
import CodeBlock from '@theme/CodeBlock';

export default function Home() {
    const {siteConfig} = useDocusaurusContext();
    return (<Layout
        description={siteConfig.tagline}>
        <HomepageHeader/>
        <main className={styles.main}>
            <h2>
                How it works
            </h2>
            {STEPS.map((step, stepIndex) => <div key={stepIndex}>
                <StepHeader index={stepIndex} step={STEPS[stepIndex]}/>
                <div className={styles.stepDetail}>
                    {step.image ? <div className={styles.stepImage}
                                       style={{gridRow: `1 / ${STEPS[stepIndex].benefits.length + 1}`}}>{step.image}</div> : <> </>}
                    {STEPS[stepIndex].benefits.map((benefit, benefitIndex) => <React.Fragment
                        key={`${stepIndex}.${benefitIndex}`}>
                        {benefit.image ? <div className={styles.benefitImage}>{benefit.image}</div> : <> </>}
                        <Feature benefit={benefit}/>
                    </React.Fragment>)}
                </div>
            </div>)}
            <div className={styles.getStartedContainer}>
                <div className={styles.getStartedCard}>
                    <h2>
                        Get started
                    </h2>
                    <p>
                        Login to create a free project and start inspecting your data
                    </p>
                    <Link
                        className="button button--primary button--lg"
                        to="https://app.observation.tools/">
                        Login
                    </Link>
                    <Link
                        className="button button--secondary button--lg"
                        to="https://docs.rs/observation-tools/">
                        Read the docs
                    </Link>
                </div>
            </div>
        </main>
    </Layout>);
}

function useWindowDimensions() {
    const isBrowser = useIsBrowser();

    function getWindowDimensions() {
        if (!isBrowser) {
            return {width: 0, height: 0};
        }
        // TODO(doug): Get size from headers for SSR
        return {
            width: window?.innerWidth || 0,
            height: window?.innerHeight || 0,
        }
    }

    const [windowDimensions, setWindowDimensions] = useState(
        getWindowDimensions()
    );
    useEffect(() => {
        function handleResize() {
            setWindowDimensions(getWindowDimensions())
        }

        handleResize();
        window.addEventListener("resize", handleResize)
        return () => window.removeEventListener("resize", handleResize)
    }, [isBrowser]);
    return windowDimensions
}

function HeaderBackground() {
    const FONT_SIZE = 32;
    const {width, height} = useWindowDimensions();
    const lines = useMemo(() => {
        let radius = FONT_SIZE;
        let lines = [];
        while (radius < Math.max(width, height)) {
            let path = "M";
            const NUM_POINTS = 32;
            for (let i = 0; i < NUM_POINTS; i++) {
                const angle = i / NUM_POINTS * Math.PI * 2;
                const x = Math.cos(angle) * radius + width / 2;
                const y = Math.sin(angle) * radius + height / 2;
                path += ` ${x} ${y}`;
            }
            path += "z";
            lines.push(path);
            radius += FONT_SIZE;
        }
        return lines;
    }, [FONT_SIZE, width, height]);
    return <>
        <svg viewBox={`0 0 ${width} ${height}`}
             aria-hidden={true}
             className={styles.heroBackground}>
            <g
                className={styles.headerBackgroundText}
            >
                {lines.map((path, index) => {
                    return <text key={index} x={-2 * FONT_SIZE + Math.sin(index) * FONT_SIZE}
                                 y={FONT_SIZE * index}
                                 style={{fontSize: FONT_SIZE}}>
                        {path}
                    </text>
                })}
            </g>
            <g
                className={styles.headerBackgroundLines}
            >
                {lines.map((path, index) => {
                    return <path key={index} stroke="context-stroke" fill="none" d={path} strokeWidth={5}/>
                })}
            </g>
        </svg>
    </>;
}

function HomepageHeader() {
    const {siteConfig} = useDocusaurusContext();
    return (<header className={clsx('hero hero--secondary', styles.heroBanner)}>
        <HeaderBackground/>
        <div className={clsx("container", styles.heroBannerContent)} style={{zIndex: 1}}>
            <h1 className="hero__title">
                Logging meets the <span className={styles.title3d}>2nd dimension</span>
            </h1>
            <h2 className="hero__subtitle">
                {siteConfig.tagline}
            </h2>
            <div className={styles.buttons}>
                <Link
                    className="button button--primary button--lg"
                    to="https://docs.rs/observation-tools/">
                    Get started
                </Link>
            </div>
        </div>
    </header>);
}

function StepHeader({index, step}) {
    return (<div className={styles.stepHeader}>
        <div className={styles.step}>
            <div className={styles.stepLine}>
            </div>
            <div className={styles.stepNumber}>{index + 1}</div>
        </div>
        <h3>
            {step.title}
        </h3>
        <p>
            {step.description}
        </p>
    </div>);
}

function Feature({benefit}) {
    const Icon = benefit.icon;
    return (<div className={styles.feature}>
        <Icon className={styles.featureIcon}/>
        <h4 className={styles.featureTitle}>
            {benefit.title}
        </h4>
        <p className={styles.featureDescription}>
            {benefit.description}
        </p>
    </div>);
}

const EXPORT_CODE = `fn generate_city() {
  let points = generate_city_outline();
  let poly = Polygon2Builder::from_points(points);
  uploader.create_object2("city_outline", poly);
}
`;

const VISUALIZE_CODE = `let image = Image2Builder::from(data);
image.set_per_pixel_transform(
  PerPixelTransformBuilder::random_distinct_color());
`;

const STEPS = [{
    title: 'Export', description: <>
        Instrument your code with our client libraries to start exporting data from anywhere in your program
    </>, image: <div className={styles.primitivesDemo}>
        <div className={styles.primitivesCode}>
            <CodeBlock
                language="rust"
                showLineNumbers={true}
            >
                {EXPORT_CODE}
            </CodeBlock>
        </div>
        <BrowserWindow className={styles.primitivesBrowserWindow}>
            <svg className={styles.primitivesImg} width="81.865mm" height="48.943mm" version="1.1"
                 viewBox="0 0 81.865 48.943" xmlns="http://www.w3.org/2000/svg">
                <defs>
                    <marker id="DotS" overflow="visible" orient="auto">
                        <path transform="matrix(.2 0 0 .2 1.48 .2)"
                              d="m-2.5-1c0 2.76-2.24 5-5 5s-5-2.24-5-5 2.24-5 5-5 5 2.24 5 5z" fill="context-stroke"
                              fillRule="evenodd" stroke="context-stroke" strokeWidth="1pt"/>
                    </marker>
                </defs>
                <g transform="translate(-56.373 -39.818)">
                    <path
                        d="m60.811 50.863 36.327-7.9284 38.001 14.804-10.405 21.109-28.165-5.8937-25.855 12.684-11.237-14.997z"
                        fill="none" markerMid="url(#DotS)" markerStart="url(#DotS)" stroke="context-stroke"
                        strokeWidth="1.865"/>
                </g>
            </svg>
        </BrowserWindow>
    </div>
    , benefits: [{
        title: 'Toolbox of existing primitives', icon: MdViewInAr, description: (<>
            Model your data with our library of existing 2d and 3d primitives, e.g. polygons, images
        </>),
    },
        {
            title: "Local or remote execution", icon: MdDeviceHub, description: <>
                Data collection is not limited to one machine. Export data from distributed systems/pipelines and
                aggregate it in a single place.
            </>
        }
    ]
}, {
    title: 'Visualize', description: <>
        View collected data in our web-based viewer. Spend time looking at your data, not deciphering text logs or
        building custom tools.
    </>, image: <div className={styles.pixelImages}>
        <div className={styles.pixelCode}>
            <CodeBlock
                language="rust"
                showLineNumbers={true}
            >
                {VISUALIZE_CODE}
            </CodeBlock>
        </div>
        <BrowserWindow className={styles.pixelImageBrowser}>
            <PixelImage useColors={true}/>
        </BrowserWindow>
    </div>,
    benefits: [{
        title: 'Library of data transformations', icon: MdHomeRepairService, description: <>
            Apply transformations to objects to make them easier to understand, e.g. recoloring images, coordinate
            transforms
        </>
    },]
}, {
    title: 'Inspect', description: <>
        Use our tools to quickly find and diagnose issues so you can get back to building
    </>,
    image: <IntersectionWindow/>,
    benefits: [{
        title: "Suite of debug tools", icon: MdBuild, description: <>
            Use our suite of analysis tools to inspect transformations, find intersections between objects, and
            more.
        </>,
    }, {
        title: "Replay state changes", icon: MdReplay, description: <>
            Go through algorithms step-by-step to see where they went wrong.
        </>,
    }]
}];

function BrowserWindow({className, children}) {
    return <>
        <div className={clsx(className, styles.browserContainer)}>
            <div className={styles.browserTopBar}>
                <div className={styles.browserUrl}>
                    https://observation.tools
                    <MdRefresh className={styles.browserRefresh}/>
                </div>
            </div>
            <div className={styles.browserContent}>
                {children}
            </div>
        </div>
    </>;
}

const INTERSECTION_STATE_COUNT = 3;

function IntersectionWindow() {
    const [index, setIndex] = useState(0);
    useEffect(() => {
        const interval = setInterval(
            () => {
                setIndex((index) => (index + 1) % INTERSECTION_STATE_COUNT);
            },
            2000,
        );
        return () => clearInterval(interval);
    }, [setIndex]);
    return <>
        <BrowserWindow className={styles.intersectionBrowserWindow}>
            <IntersectionWindowImage index={index}/>
            <input type="range" min="0" max={INTERSECTION_STATE_COUNT - 1} value={index} readOnly disabled={true}/>
        </BrowserWindow>
    </>;
}

function IntersectionWindowImage({index}) {
    switch (index) {
        default:
        case 0:
            return <>
                <svg width="80mm" height="50mm" version="1.1" viewBox="0 0 80 50" xmlns="http://www.w3.org/2000/svg"
                     className={styles.intersectionImage}>
                    <g transform="translate(-56.373 -39.818)">
                        <path transform="matrix(.12707 .15144 -.15243 .1279 102.86 46.354)"
                              d="m171.17 115.37-164.1-53.534 128.41-115.35z" fill="context-fill" stroke="context-stroke"
                              strokeWidth="5.042"/>
                        <rect transform="rotate(70)" x="78.265" y="-73.028" width="10.826" height="32.594"
                              fill="context-stroke" stroke="context-stroke"/>
                        <circle cx="99.176" cy="58.714" r="1.6882" fill="#f00" stroke="none"/>
                        <text transform="matrix(0.26458333,0,0,0.26458333,60.770254,40.082271)" fill="#f00"
                              stroke="#f00">
                            <tspan
                                x="147.47266"
                                y="50.434124"
                            >
                                (127,45)
                            </tspan>
                        </text>
                    </g>
                </svg>
            </>;
        case 1:
            return <>
                <svg width="80mm" height="50mm" version="1.1" viewBox="0 0 80 50" xmlns="http://www.w3.org/2000/svg"
                     className={styles.intersectionImage}>
                    <g transform="translate(-56.373 -39.818)">
                        <path transform="matrix(.027513 .19577 -.19705 .027693 114.61 45.793)"
                              d="m171.17 115.37-164.1-53.534 128.41-115.35z" strokeWidth="5.042"/>
                        <rect transform="rotate(70)" x="78.265" y="-73.028" width="10.826" height="32.594"/>
                    </g>
                </svg>
            </>;
        case 2:
            return <>
                <svg width="80mm" height="50mm" version="1.1" viewBox="0 0 80 50" xmlns="http://www.w3.org/2000/svg"
                     className={styles.intersectionImage}>
                    <g transform="translate(-56.373 -39.818)">
                        <path transform="matrix(-.074056 .1833 -.18449 -.07454 124.33 51.022)"
                              d="m171.17 115.37-164.1-53.534 128.41-115.35z" strokeWidth="5.042"/>
                        <rect transform="rotate(70)" x="78.265" y="-73.028" width="10.826" height="32.594"/>
                    </g>
                </svg>
            </>;
    }
}

function PixelImage({}) {
    const width = 10;
    const height = 10;
    const data = useMemo(() => {
        const data = Array(width * height).fill();
        for (let x = 0; x < width; x++) {
            for (let y = 0; y < height; y++) {
                const index = x + y * width;
                let value = 0;
                if (x > 1 && x < 4) {
                    if (y > 4 && y < 8) {
                        value = 1;
                    }
                }
                const distance = Math.sqrt(Math.pow((x - width), 2) + Math.pow((y - height), 2));
                const radius = width / 2;
                if (distance < radius) {
                    value = 2;
                }
                if (y < x / 2) {
                    value = 3;
                }
                data[index] = value;
            }
        }
        return data;
    }, [width, height]);
    return <div className={styles.pixelImage}
                aria-hidden={true}>
        {data.map((value, i) => {
            return <div className={styles.pixel} key={i} data-color-index={value}>
                {value}
            </div>;
        })}
    </div>;
}