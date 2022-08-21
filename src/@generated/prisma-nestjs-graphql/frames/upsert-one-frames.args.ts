import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesWhereUniqueInput } from './frames-where-unique.input';
import { Type } from 'class-transformer';
import { FramesCreateInput } from './frames-create.input';
import { FramesUpdateInput } from './frames-update.input';

@ArgsType()
export class UpsertOneFramesArgs {

    @Field(() => FramesWhereUniqueInput, {nullable:false})
    @Type(() => FramesWhereUniqueInput)
    where!: FramesWhereUniqueInput;

    @Field(() => FramesCreateInput, {nullable:false})
    @Type(() => FramesCreateInput)
    create!: FramesCreateInput;

    @Field(() => FramesUpdateInput, {nullable:false})
    @Type(() => FramesUpdateInput)
    update!: FramesUpdateInput;
}
