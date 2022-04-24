import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Frames_Type } from './frames-type.enum';
import { NestedEnumFrames_TypeFilter } from './nested-enum-frames-type-filter.input';

@InputType()
export class EnumFrames_TypeFilter {

    @Field(() => Frames_Type, {nullable:true})
    equals?: keyof typeof Frames_Type;

    @Field(() => [Frames_Type], {nullable:true})
    in?: Array<keyof typeof Frames_Type>;

    @Field(() => [Frames_Type], {nullable:true})
    notIn?: Array<keyof typeof Frames_Type>;

    @Field(() => NestedEnumFrames_TypeFilter, {nullable:true})
    not?: NestedEnumFrames_TypeFilter;
}
