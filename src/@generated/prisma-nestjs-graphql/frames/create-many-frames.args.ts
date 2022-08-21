import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesCreateManyInput } from './frames-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyFramesArgs {

    @Field(() => [FramesCreateManyInput], {nullable:false})
    @Type(() => FramesCreateManyInput)
    data!: Array<FramesCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
