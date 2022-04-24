import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Frames_Type } from './frames-type.enum';

@InputType()
export class NestedEnumFrames_TypeFilter {

    @Field(() => Frames_Type, {nullable:true})
    equals?: keyof typeof Frames_Type;

    @Field(() => [Frames_Type], {nullable:true})
    in?: Array<keyof typeof Frames_Type>;

    @Field(() => [Frames_Type], {nullable:true})
    notIn?: Array<keyof typeof Frames_Type>;

    @Field(() => NestedEnumFrames_TypeFilter, {nullable:true})
    not?: NestedEnumFrames_TypeFilter;
}
