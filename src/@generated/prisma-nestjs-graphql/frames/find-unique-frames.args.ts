import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesWhereUniqueInput } from './frames-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class FindUniqueFramesArgs {

    @Field(() => FramesWhereUniqueInput, {nullable:false})
    @Type(() => FramesWhereUniqueInput)
    where!: FramesWhereUniqueInput;
}
