import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesCreateManyInput } from './frames-create-many.input';

@ArgsType()
export class CreateManyFramesArgs {

    @Field(() => [FramesCreateManyInput], {nullable:false})
    data!: Array<FramesCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
