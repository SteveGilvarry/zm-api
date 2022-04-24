import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_Orientation } from '../monitors/monitors-orientation.enum';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumMonitors_OrientationFilter } from './nested-enum-monitors-orientation-filter.input';

@InputType()
export class NestedEnumMonitors_OrientationWithAggregatesFilter {

    @Field(() => Monitors_Orientation, {nullable:true})
    equals?: keyof typeof Monitors_Orientation;

    @Field(() => [Monitors_Orientation], {nullable:true})
    in?: Array<keyof typeof Monitors_Orientation>;

    @Field(() => [Monitors_Orientation], {nullable:true})
    notIn?: Array<keyof typeof Monitors_Orientation>;

    @Field(() => NestedEnumMonitors_OrientationWithAggregatesFilter, {nullable:true})
    not?: NestedEnumMonitors_OrientationWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumMonitors_OrientationFilter, {nullable:true})
    _min?: NestedEnumMonitors_OrientationFilter;

    @Field(() => NestedEnumMonitors_OrientationFilter, {nullable:true})
    _max?: NestedEnumMonitors_OrientationFilter;
}
