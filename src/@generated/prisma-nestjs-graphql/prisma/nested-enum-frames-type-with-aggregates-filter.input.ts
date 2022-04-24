import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Frames_Type } from './frames-type.enum';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumFrames_TypeFilter } from './nested-enum-frames-type-filter.input';

@InputType()
export class NestedEnumFrames_TypeWithAggregatesFilter {

    @Field(() => Frames_Type, {nullable:true})
    equals?: keyof typeof Frames_Type;

    @Field(() => [Frames_Type], {nullable:true})
    in?: Array<keyof typeof Frames_Type>;

    @Field(() => [Frames_Type], {nullable:true})
    notIn?: Array<keyof typeof Frames_Type>;

    @Field(() => NestedEnumFrames_TypeWithAggregatesFilter, {nullable:true})
    not?: NestedEnumFrames_TypeWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumFrames_TypeFilter, {nullable:true})
    _min?: NestedEnumFrames_TypeFilter;

    @Field(() => NestedEnumFrames_TypeFilter, {nullable:true})
    _max?: NestedEnumFrames_TypeFilter;
}
