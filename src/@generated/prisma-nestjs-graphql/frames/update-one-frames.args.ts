import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesUpdateInput } from './frames-update.input';
import { Type } from 'class-transformer';
import { FramesWhereUniqueInput } from './frames-where-unique.input';

@ArgsType()
export class UpdateOneFramesArgs {

    @Field(() => FramesUpdateInput, {nullable:false})
    @Type(() => FramesUpdateInput)
    data!: FramesUpdateInput;

    @Field(() => FramesWhereUniqueInput, {nullable:false})
    @Type(() => FramesWhereUniqueInput)
    where!: FramesWhereUniqueInput;
}
